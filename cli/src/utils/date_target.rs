use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum DateTarget {
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

impl FromStr for DateTarget {
    type Err = anyhow::Error;

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
                Err(e) => anyhow::bail!("Invalid date target: {}", e),
            },
        }
    }
}

impl ToString for DateTarget {
    fn to_string(&self) -> String {
        match self {
            DateTarget::All => "all".to_string(),
            DateTarget::Past => "past".to_string(),
            DateTarget::Future => "future".to_string(),
            DateTarget::Today => "today".to_string(),
            DateTarget::Yesterday => "yesterday".to_string(),
            DateTarget::LastWeek => "last week".to_string(),
            DateTarget::LastMonth => "last month".to_string(),
            DateTarget::NextWeek => "next week".to_string(),
            DateTarget::NextMonth => "next month".to_string(),
            DateTarget::Specific(dt) => dt.to_string(),
        }
    }
}

impl Serialize for DateTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]

    use chrono::Datelike;

    use super::*;

    #[test]
    fn test_date_target_parsing() {
        assert_eq!("".parse::<DateTarget>().unwrap(), DateTarget::All);
        assert_eq!("all".parse::<DateTarget>().unwrap(), DateTarget::All);
        assert_eq!("past".parse::<DateTarget>().unwrap(), DateTarget::Past);
        assert_eq!("future".parse::<DateTarget>().unwrap(), DateTarget::Future);
        assert_eq!("today".parse::<DateTarget>().unwrap(), DateTarget::Today);
        assert_eq!(
            "yesterday".parse::<DateTarget>().unwrap(),
            DateTarget::Yesterday
        );
        assert_eq!(
            "last week".parse::<DateTarget>().unwrap(),
            DateTarget::LastWeek
        );
        assert_eq!(
            "last month".parse::<DateTarget>().unwrap(),
            DateTarget::LastMonth
        );
        assert_eq!(
            "next week".parse::<DateTarget>().unwrap(),
            DateTarget::NextWeek
        );
        assert_eq!(
            "next month".parse::<DateTarget>().unwrap(),
            DateTarget::NextMonth
        );
    }

    #[test]
    fn test_date_target_tostring() {
        assert_eq!(DateTarget::All.to_string(), "all");
        assert_eq!(DateTarget::Past.to_string(), "past");
        assert_eq!(DateTarget::Future.to_string(), "future");
        assert_eq!(DateTarget::Today.to_string(), "today");
        assert_eq!(DateTarget::Yesterday.to_string(), "yesterday");
        assert_eq!(DateTarget::LastWeek.to_string(), "last week");
        assert_eq!(DateTarget::LastMonth.to_string(), "last month");
        assert_eq!(DateTarget::NextWeek.to_string(), "next week");
        assert_eq!(DateTarget::NextMonth.to_string(), "next month");
    }

    #[test]
    fn test_specific_date_parsing() {
        let date = DateTarget::from_str("2024-03-16").unwrap();
        match date {
            DateTarget::Specific(dt) => {
                assert_eq!(dt.year(), 2024);
                assert_eq!(dt.month(), 3);
                assert_eq!(dt.day(), 16);
            }
            _ => panic!("Expected Specific date"),
        }
    }

    #[test]
    fn test_specific_date_parsing_panic() {
        let err = DateTarget::from_str("xxx").err();
        match err {
            Some(e) => assert_eq!(
                e.to_string(),
                "Invalid date target: input contains invalid characters"
            ),
            None => panic!("Expected error"),
        }

        let err = DateTarget::from_str("20-20-20").err();
        match err {
            Some(e) => assert_eq!(e.to_string(), "Invalid date target: input is out of range"),
            None => panic!("Expected error"),
        }
    }
}
