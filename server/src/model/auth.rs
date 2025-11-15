use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeviceCodeRequest {
    pub device_code: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeviceStatusResponse {
    pub access_token: String,
}

pub enum ChallengeResult {
    Success(String),
    NoChallenge,
    Pending,
}

use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Clone)]
#[expect(dead_code)]
pub struct DeviceAuth {
    pub id: i64,
    pub expire_date: DateTime<Utc>,
    pub device_code: String,
    pub token: Option<String>,
}

#[derive(Debug)]
pub struct DeviceAuthEntity {
    pub id: i64,
    pub expire_date: NaiveDateTime,
    pub device_code: String,
    pub token: Option<String>,
}

impl TryFrom<DeviceAuthEntity> for DeviceAuth {
    type Error = String;

    fn try_from(value: DeviceAuthEntity) -> Result<Self, Self::Error> {
        Ok(DeviceAuth {
            id: value.id,
            expire_date: DateTime::from_naive_utc_and_offset(value.expire_date, Utc),
            device_code: value.device_code,
            token: value.token,
        })
    }
}
