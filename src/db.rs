use rusqlite::{Result, params};

const CREATE_TABLE_QUERY: &str = "
CREATE TABLE IF NOT EXISTS plants (
    slug TEXT PRIMARY KEY NOT NULL, 
    name TEXT NOT NULL, 
    interval_days INTEGER NOT NULL, 
    last_watered_at INTEGER
);
";

pub fn create_table(connection: &rusqlite::Connection) -> Result<()> {
    connection.execute(CREATE_TABLE_QUERY, params![])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_table_called_repeated_times() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();
        create_table(&conn).unwrap();
    }
}
