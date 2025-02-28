use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, de};
use utoipa::IntoParams;

const DEFAULT_PAGINATION_OFFSET: i64 = 0;
const DEFAULT_PAGINATION_LIMIT: i64 = 100;

#[derive(Debug, Deserialize, IntoParams)]
#[allow(dead_code)]
pub struct Pagination {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    limit: Option<i64>,
    offset: Option<i64>,
}

impl Pagination {
    pub fn get_offset(&self) -> i64 {
        self.offset.unwrap_or(DEFAULT_PAGINATION_OFFSET)
    }

    pub fn get_limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_PAGINATION_LIMIT)
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            limit: Some(0),
            offset: Some(10),
        }
    }
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
