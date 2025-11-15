use std::str::FromStr;

use crate::{errors::DateFilterError, util::DateTimeWrapper};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Note {
    pub id: i64,
    pub content: String,
    pub tags: Vec<String>,
    pub user_id: i64,
    #[schemars(with = "DateTimeWrapper")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schemars(with = "DateTimeWrapper")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
pub struct NoteEntity {
    pub id: i64,
    pub content: String,
    pub tags: String,
    pub user_id: i64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl TryFrom<NoteEntity> for Note {
    type Error = String;

    fn try_from(val: NoteEntity) -> Result<Self, String> {
        let created_at = val
            .created_at
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .ok_or("created_at timestamp is missing".to_string())?;

        let updated_at = val
            .updated_at
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .ok_or("updated_at timestamp is missing".to_string())?;

        Ok(Note {
            id: val.id,
            content: val.content,
            tags: val.tags.split(",").map(|s| s.to_string()).collect(),
            user_id: val.user_id,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateNoteRequest {
    pub content: String,
    pub tags: Vec<String>,
}

pub fn parse_date_filter(filter: &str) -> DateFilter {
    match filter.to_lowercase().as_str() {
        "all" | "" => DateFilter::All,
        "past" => DateFilter::Past,
        "future" => DateFilter::Future,
        "today" => DateFilter::Today,
        "yesterday" => DateFilter::Yesterday,
        "last week" => DateFilter::LastWeek,
        "last month" => DateFilter::LastMonth,
        "next week" => DateFilter::NextWeek,
        "next month" => DateFilter::NextMonth,
        _ => match NaiveDate::parse_from_str(filter, "%Y-%m-%d") {
            Ok(dt) => DateFilter::Specific(dt),
            Err(_) => DateFilter::All,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum DateFilter {
    All,
    Past,
    Future,
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
    NextWeek,
    NextMonth,
    Specific(NaiveDate),
}

impl FromStr for DateFilter {
    type Err = DateFilterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" | "" => Ok(Self::All),
            "past" => Ok(Self::Past),
            "future" => Ok(Self::Future),
            "today" => Ok(Self::Today),
            "yesterday" => Ok(Self::Yesterday),
            "last week" => Ok(Self::LastWeek),
            "last month" => Ok(Self::LastMonth),
            "next week" => Ok(Self::NextWeek),
            "next month" => Ok(Self::NextMonth),
            _ => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                Ok(dt) => Ok(Self::Specific(dt)),
                Err(e) => Err(DateFilterError::ParseError(format!(
                    "Error while parsing '{}': {}",
                    s.to_string(),
                    e
                ))),
            },
        }
    }
}
