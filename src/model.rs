use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Plant {
    pub slug: String,
    pub name: String,
    pub interval_days: i64,
    pub last_watered_at: Option<i64>, // unix seconds
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
