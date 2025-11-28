pub mod models;

use crate::models::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct UsosResponse {
    pub data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum UsosError {
    #[error("")]
    UrlEncode(#[from] serde_urlencoded::ser::Error),

    #[error("")]
    UrlParse(#[from] url::ParseError),

    #[error("")]
    JsonParse(#[from] serde_json::Error),
}

pub trait UsosClient {
    fn send(&self, request: Url) -> Result<UsosResponse, UsosError>;
}

pub struct Usos<C>
where
    C: UsosClient,
{
    client: C,
    base: String,
}

impl<C> Usos<C>
where
    C: UsosClient,
{
    pub fn new(client: C, base: Url) -> Self {
        Self {
            client,
            base: base.to_string(),
        }
    }

    pub fn timetable_user(
        &self,
        start: Option<TimetableUserStart>,
        days: Option<u8>,
        fields: Option<&[&str]>,
    ) -> Result<UsosTimetableUser, UsosError> {
        let mut params = HashMap::new();
        if let Some(start) = start {
            params.insert("start".to_owned(), start.to_string());
        }
        if let Some(days) = days {
            params.insert("days".to_owned(), days.to_string());
        }
        if let Some(fields) = fields {
            let fields_combined = fields.join("|");
            params.insert("fields".to_owned(), fields_combined);
        }
        let request_url = Url::from_str(&format!(
            "{}/services/tt/user{}",
            self.base,
            if params.is_empty() {
                String::new()
            } else {
                serde_urlencoded::to_string(&params)?
            }
        ))?;
        Ok(serde_json::from_slice::<UsosTimetableUser>(
            &self.client.send(request_url)?.data,
        )?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    struct TestUsos {}
    impl TestUsos {
        fn new() -> Self {
            Self {}
        }
    }

    impl UsosClient for TestUsos {
        fn send(&self, request: Url) -> Result<UsosResponse, UsosError> {
            Ok(UsosResponse { data: Vec::new() })
        }
    }

    #[test]
    fn test() {
        let mut usos = Usos::new(
            TestUsos::new(),
            Url::from_str("https://127.0.0.1/usos").unwrap(),
        );
    }
}
