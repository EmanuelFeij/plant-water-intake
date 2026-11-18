use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, PartialEq, Serialize)]
pub struct Plant {
    pub slug: String,
    pub name: String,
    pub interval_days: i64,
    pub last_watered_at: Option<i64>, // unix seconds
}

impl Plant {
    pub fn is_overdue(&self, now: i64) -> bool {
        self.last_watered_at
            .is_none_or(|lwa| now > ((self.interval_days * 86400) + lwa))
    }
}

#[derive(Debug, PartialEq)]
pub enum MarkedWateredOutcome {
    Watered(String),
    WateredRecently(String), // at max 10 minutes prior
    NotFound,
}

#[derive(Debug, PartialEq)]
pub enum InsertOutcome {
    Created,
    PlantAlreadyExists,
}

#[derive(Debug, PartialEq)]
pub struct Elapsed {
    days: i64,
    hours: i64,
}

impl Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let days: Option<String> = match self.days {
            0 => None,
            1 => Some("1 Day".to_string()),
            _ => Some(format!("{} Days", self.days)),
        };
        let hours: Option<String> = match self.hours {
            0 => None,
            1 => Some("1 Hour".to_string()),
            _ => Some(format!("{} Hours", self.hours)),
        };

        match (days, hours) {
            (None, Some(h)) => write!(f, "{}", h),
            (Some(d), None) => write!(f, "{}", d),
            (Some(ds), Some(hs)) => write!(f, "{} and {}", ds, hs),
            (None, None) => write!(f, "Less Than 1 Hour"),
        }
    }
}

pub fn time_since(from: i64, now: i64) -> Elapsed {
    let elapsed = now - from;
    let hours = (elapsed % 86400) / 3600;
    let days = elapsed / 86400;

    Elapsed { days, hours }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn days_since_success() {
        let secs: i64 = 56 * 3600;

        let elapsed = time_since(0, secs);
        assert_eq!(elapsed.days, 2);
        assert_eq!(elapsed.hours, 8);

        assert_eq!(elapsed.to_string(), "2 Days and 8 Hours")
    }

    #[test]
    fn days_since_success_24h() {
        let secs: i64 = 24 * 3600;

        let elapsed = time_since(0, secs);
        assert_eq!(elapsed.days, 1);
        assert_eq!(elapsed.hours, 0);

        assert_eq!(elapsed.to_string(), "1 Day");
    }

    #[test]
    fn days_since_success_23h59m() {
        let secs: i64 = (23 * 3600) + 3599;

        let elapsed = time_since(0, secs);
        assert_eq!(elapsed.days, 0);
        assert_eq!(elapsed.hours, 23);

        assert_eq!(elapsed.to_string(), "23 Hours")
    }

    #[test]
    fn days_since_success_less_than_an_hour() {
        let elapsed = time_since(0, 59 * 60);

        assert_eq!(elapsed.days, 0);
        assert_eq!(elapsed.hours, 0);

        assert_eq!(elapsed.to_string(), "Less Than 1 Hour");
    }

    #[test]
    fn is_overdue_several_cases() {
        let plant = Plant {
            slug: String::from("WhiteLily"),
            name: String::from("Lily"),
            interval_days: 6,
            last_watered_at: Some(100),
        };

        let plant2 = Plant {
            slug: String::from("rose"),
            name: String::from("Rose"),
            interval_days: 6,
            last_watered_at: None,
        };

        assert!(!plant.is_overdue(518500));
        assert!(!plant.is_overdue(500000));
        assert!(plant.is_overdue(518700));
        assert!(plant2.is_overdue(0));
    }
}
