use std::{collections::HashMap, error::Error as StdError, fmt::Display, sync::LazyLock};

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

use crate::libs::http::errors::{ServerError, ServiceError};

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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum JwtError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InvalidSignature,
    InvalidClaims,
    ExpiredSignature,
}

impl Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl StdError for JwtError {}

impl ServiceError for JwtError {
    fn status(&self) -> StatusCode {
        JWT_ERRORS
            .get(self)
            .map_or(StatusCode::INTERNAL_SERVER_ERROR, |e| e.code)
    }

    fn message(&self) -> String {
        JWT_ERRORS
            .get(self)
            .map_or("unknown error".to_string(), |e| e.error_message.to_string())
    }

    fn field_as_string(&self) -> String {
        JWT_ERRORS
            .get(self)
            .map_or("UNKNOWN_ERROR".to_string(), |e| e.error_type.to_string())
    }
}

struct FullError {
    error_type: String,
    error_message: String,
    code: StatusCode,
}

static JWT_ERRORS: LazyLock<HashMap<JwtError, FullError>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert(
        JwtError::WrongCredentials,
        FullError {
            error_type: "AUTH_WRONG_CREDENTIALS".to_string(),
            error_message: "wrong credentials".to_string(),
            code: StatusCode::UNAUTHORIZED,
        },
    );

    map.insert(
        JwtError::MissingCredentials,
        FullError {
            error_type: "AUTH_MISSING_CREDENTIALS".to_string(),
            error_message: "missing credentials".to_string(),
            code: StatusCode::BAD_REQUEST,
        },
    );

    map.insert(
        JwtError::TokenCreation,
        FullError {
            error_type: "AUTH_TOKEN_CREATION".to_string(),
            error_message: "token creation".to_string(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        },
    );

    map.insert(
        JwtError::InvalidToken,
        FullError {
            error_type: "AUTH_INVALID_TOKEN".to_string(),
            error_message: "invalid token".to_string(),
            code: StatusCode::BAD_REQUEST,
        },
    );

    map.insert(
        JwtError::InvalidSignature,
        FullError {
            error_type: "AUTH_INVALID_SIGNATURE".to_string(),
            error_message: "invalid signature".to_string(),
            code: StatusCode::UNAUTHORIZED,
        },
    );

    map.insert(
        JwtError::InvalidClaims,
        FullError {
            error_type: "AUTH_INVALID_CLAIMS".to_string(),
            error_message: "invalid claims".to_string(),
            code: StatusCode::UNAUTHORIZED,
        },
    );

    map.insert(
        JwtError::ExpiredSignature,
        FullError {
            error_type: "AUTH_EXPIRED_SIGNATURE".to_string(),
            error_message: "expired signature".to_string(),
            code: StatusCode::UNAUTHORIZED,
        },
    );

    map
});
