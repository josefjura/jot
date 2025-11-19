use std::str::FromStr;

use chrono::{Days, Local, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DateSource {
    Today,
    Yesterday,
    Tomorrow,
    Specific(NaiveDate),
}

impl FromStr for DateSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "today" => Ok(Self::Today),
            "yesterday" => Ok(Self::Yesterday),
            "tomorrow" => Ok(Self::Tomorrow),
            _ => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                Ok(dt) => Ok(Self::Specific(dt)),
                Err(e) => anyhow::bail!("Invalid date target: {}", e),
            },
        }
    }
}

// Custom deserializer implementation
impl<'de> Deserialize<'de> for DateSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "today" => Ok(DateSource::Today),
            "yesterday" => Ok(DateSource::Yesterday),
            "tomorrow" => Ok(DateSource::Tomorrow),
            date => {
                let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map_err(serde::de::Error::custom)?;
                Ok(DateSource::Specific(parsed_date))
            }
        }
    }
}

impl ToString for DateSource {
    fn to_string(&self) -> String {
        self.to_date().format("%Y-%m-%d").to_string()
    }
}

impl Default for DateSource {
    fn default() -> Self {
        Self::Today
    }
}

impl From<DateSource> for NaiveDate {
    fn from(ds: DateSource) -> Self {
        ds.to_date()
    }
}

impl DateSource {
    pub fn to_date(&self) -> NaiveDate {
        match self {
            Self::Today => Local::now().date_naive(),
            Self::Yesterday => Local::now().date_naive() - Days::new(1),
            Self::Tomorrow => Local::now().date_naive() + Days::new(1),
            Self::Specific(dt) => *dt,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]

    use chrono::Datelike;

    use super::*;

    #[test]
    fn test_date_target_parsing() {
        assert_eq!("today".parse::<DateSource>().unwrap(), DateSource::Today);
        assert_eq!(
            "yesterday".parse::<DateSource>().unwrap(),
            DateSource::Yesterday
        );
        assert_eq!(
            "tomorrow".parse::<DateSource>().unwrap(),
            DateSource::Tomorrow
        );
    }

    #[test]
    fn test_specific_date_parsing() {
        let date = DateSource::from_str("2024-03-16").unwrap();
        match date {
            DateSource::Specific(dt) => {
                assert_eq!(dt.year(), 2024);
                assert_eq!(dt.month(), 3);
                assert_eq!(dt.day(), 16);
            }
            _ => panic!("Expected Specific date"),
        }
    }

    #[test]
    fn test_specific_date_parsing_panic() {
        let err = DateSource::from_str("xxx").err();
        match err {
            Some(e) => assert_eq!(
                e.to_string(),
                "Invalid date target: input contains invalid characters"
            ),
            None => panic!("Expected error"),
        }

        let err = DateSource::from_str("20-20-20").err();
        match err {
            Some(e) => assert_eq!(e.to_string(), "Invalid date target: input is out of range"),
            None => panic!("Expected error"),
        }
    }
}
