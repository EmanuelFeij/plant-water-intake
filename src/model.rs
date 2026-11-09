#[derive(Debug)]
pub struct Plant {
    pub slug: String,
    pub name: String,
    pub interval_days: i64,
    pub last_watered_at: Option<i64>, // unix seconds
}
