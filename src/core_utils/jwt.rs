use std::{error::Error as StdError, fmt::Display, sync::LazyLock};

use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
    get_current_timestamp,
};
use serde::{Deserialize, Serialize};

use crate::core_utils::errors::ServerError;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("env var JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn expiry(secs_valid_for: u64) -> u64 {
    get_current_timestamp() + secs_valid_for
}

pub fn encode_token(claims: &Claims) -> Result<String, JwtError> {
    encode(&Header::default(), &claims, &KEYS.encoding).map_err(|_| JwtError::TokenCreation)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}

impl Claims {
    pub fn get_user_id(&self) -> i32 {
        self.sub.parse::<i32>().unwrap()
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| JwtError::InvalidToken)?;

        let token_data =
            match decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default()) {
                Ok(data) => data,
                Err(err) => match err.kind() {
                    ErrorKind::ExpiredSignature => Err(JwtError::ExpiredSignature)?,
                    ErrorKind::InvalidToken => Err(JwtError::InvalidToken)?,
                    ErrorKind::InvalidSignature => Err(JwtError::InvalidSignature)?,
                    ErrorKind::Json(_) => Err(JwtError::InvalidClaims)?,
                    _ => Err(JwtError::InvalidToken)?,
                },
            };

        Ok(token_data.claims)
    }
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sub: {}", self.sub)
    }
}

#[derive(Debug)]
pub enum JwtError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InvalidSignature,
    InvalidClaims,
    ExpiredSignature,
}

impl JwtError {
    pub fn field_as_string(&self) -> String {
        match self {
            JwtError::WrongCredentials => String::from("WRONG_CREDENTIALS"),
            JwtError::MissingCredentials => String::from("MISSING_CREDENTIALS"),
            JwtError::TokenCreation => String::from("TOKEN_CREATION"),
            JwtError::InvalidToken => String::from("INVALID_TOKEN"),
            JwtError::InvalidSignature => String::from("INVALID_SIGNATURE"),
            JwtError::InvalidClaims => String::from("INVALID_CLAIMS"),
            JwtError::ExpiredSignature => String::from("EXPIRED_SIGNATURE"),
        }
    }

    pub fn to_status_code(&self) -> StatusCode {
        match self {
            JwtError::WrongCredentials => StatusCode::UNAUTHORIZED,
            JwtError::MissingCredentials => StatusCode::BAD_REQUEST,
            JwtError::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
            JwtError::InvalidToken => StatusCode::BAD_REQUEST,
            JwtError::InvalidSignature => StatusCode::UNAUTHORIZED,
            JwtError::InvalidClaims => StatusCode::UNAUTHORIZED,
            JwtError::ExpiredSignature => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn to_message(&self) -> String {
        match self {
            JwtError::WrongCredentials => String::from("wrong credentials"),
            JwtError::MissingCredentials => String::from("missing credentials"),
            JwtError::TokenCreation => String::from("token creation"),
            JwtError::InvalidToken => String::from("invalid token"),
            JwtError::InvalidSignature => String::from("invalid signature"),
            JwtError::InvalidClaims => String::from("invalid claims"),
            JwtError::ExpiredSignature => String::from("expired signature"),
        }
    }
}

impl Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_message())
    }
}

impl StdError for JwtError {}
