use rusqlite::{OptionalExtension, Result, Row, params};

use crate::model::{MarkedWateredOutcome, Plant};

const CREATE_TABLE_QUERY: &str = "
CREATE TABLE IF NOT EXISTS plants (
    slug TEXT PRIMARY KEY NOT NULL, 
    name TEXT NOT NULL, 
    interval_days INTEGER NOT NULL, 
    last_watered_at INTEGER DEFAULT NULL
);
";

const GET_PLANT_QUERY: &str = "
SELECT slug, name, interval_days, last_watered_at 
FROM plants
WHERE slug = ?;
";

const INSERT_PLANT_QUERY: &str = "
INSERT INTO plants 
VALUES ( ?, ?, ?, ?);
";

const GET_PLANTS_QUERY: &str = "
SELECT slug, name, interval_days, last_watered_at 
FROM plants
";

const MARK_WATERED_QUERY: &str = "
UPDATE plants SET last_watered_at = ? 
WHERE slug = ? AND ((last_watered_at IS NULL) OR (last_watered_at + 600 <= ?));
";

pub fn create_table(connection: &rusqlite::Connection) -> Result<()> {
    connection.execute(CREATE_TABLE_QUERY, params![])?;
    Ok(())
}

pub fn get_plant(connection: &rusqlite::Connection, plant_slug: &str) -> Result<Option<Plant>> {
    connection
        .query_one(GET_PLANT_QUERY, params![plant_slug], row_to_plant)
        .optional() // turns Err(QueryReturnedNoRows) into Ok(None)
}

pub fn get_plants(connection: &rusqlite::Connection) -> Result<Vec<Plant>> {
    let mut stmt = connection.prepare(GET_PLANTS_QUERY)?;

    let plants = stmt
        .query_map(params![], row_to_plant)?
        .collect::<Result<Vec<Plant>>>()?;

    Ok(plants)
}

pub fn insert_plant(connection: &rusqlite::Connection, plant: &Plant) -> Result<()> {
    connection.execute(
        INSERT_PLANT_QUERY,
        params![
            plant.slug,
            plant.name,
            plant.interval_days,
            plant.last_watered_at
        ],
    )?;

    Ok(())
}

// TODO: add the 10 minutes lock
pub fn mark_watered(
    connection: &rusqlite::Connection,
    plant_slug: &str,
    unix_now_in_seconds: i64,
) -> Result<MarkedWateredOutcome> {
    match connection.execute(
        MARK_WATERED_QUERY,
        params![unix_now_in_seconds, plant_slug, unix_now_in_seconds],
    )? {
        0 => match get_plant(connection, plant_slug)? {
            Some(_) => Ok(MarkedWateredOutcome::WateredRecently),
            None => Ok(MarkedWateredOutcome::NotFound),
        },
        _ => Ok(MarkedWateredOutcome::Watered),
    }
}

fn row_to_plant(row: &Row) -> Result<Plant> {
    Ok(Plant {
        slug: row.get(0)?,
        name: row.get(1)?,
        interval_days: row.get(2)?,
        last_watered_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::model::MarkedWateredOutcome;

    use super::*;

    #[test]
    fn create_table_called_repeated_times() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();
        create_table(&conn).unwrap();
    }

    #[test]
    fn insert_plant_success() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let plant1 = Plant {
            slug: String::from("WhiteLily"),
            name: String::from("Lily"),
            interval_days: 6,
            last_watered_at: Some(100),
        };
        insert_plant(&conn, &plant1).unwrap();

        let plant2 = Plant {
            slug: String::from("RoseLily"),
            name: String::from("Tulip"),
            interval_days: 6,
            last_watered_at: None,
        };
        insert_plant(&conn, &plant2).unwrap();

        let plants = get_plants(&conn).unwrap();
        assert_eq!(plants.len(), 2);

        assert!(plants.contains(&plant1));
        assert!(plants.contains(&plant2));
    }

    #[test]
    fn get_plant_not_found() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let response = get_plant(&conn, "something").unwrap();
        assert_eq!(response, None);
    }

    #[test]
    fn get_plant_success() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let plant1 = Plant {
            slug: String::from("WhiteLily"),
            name: String::from("Lily"),
            interval_days: 6,
            last_watered_at: Some(100),
        };
        insert_plant(&conn, &plant1).unwrap();

        let response = get_plant(&conn, &plant1.slug).unwrap();
        assert_eq!(response, Some(plant1));
    }

    #[test]
    fn marked_watered_success_first_watered() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let plant1 = Plant {
            slug: String::from("WhiteLily"),
            name: String::from("Lily"),
            interval_days: 6,
            last_watered_at: None,
        };
        insert_plant(&conn, &plant1).unwrap();

        let res = mark_watered(&conn, &plant1.slug, 1000).unwrap();
        assert_eq!(res, MarkedWateredOutcome::Watered);

        let returned_plant = get_plant(&conn, &plant1.slug).unwrap().unwrap();

        assert_eq!(returned_plant.last_watered_at, Some(1000));
    }

    #[test]
    fn marked_watered_success_update() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let plant1 = Plant {
            slug: String::from("WhiteLily"),
            name: String::from("Lily"),
            interval_days: 6,
            last_watered_at: None,
        };
        insert_plant(&conn, &plant1).unwrap();

        mark_watered(&conn, &plant1.slug, 1000).unwrap();

        let returned_plant = get_plant(&conn, &plant1.slug).unwrap().unwrap();

        assert_eq!(returned_plant.last_watered_at, Some(1000));

        mark_watered(&conn, &plant1.slug, 1000 + 601).unwrap();

        let returned_plant = get_plant(&conn, &plant1.slug).unwrap().unwrap();

        assert_eq!(returned_plant.last_watered_at, Some(1000 + 601));
    }

    #[test]
    fn marked_watered_not_found() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let res = mark_watered(&conn, "some_slug", 1000).unwrap();

        assert_eq!(res, MarkedWateredOutcome::NotFound);
    }

    #[test]
    fn marked_watered_double_tap() {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");

        create_table(&conn).unwrap();

        let plant1 = Plant {
            slug: String::from("WhiteLily"),
            name: String::from("Lily"),
            interval_days: 6,
            last_watered_at: None,
        };
        insert_plant(&conn, &plant1).unwrap();

        mark_watered(&conn, &plant1.slug, 1000).unwrap();

        let res = mark_watered(&conn, &plant1.slug, 1000 + 300).unwrap();

        assert_eq!(res, MarkedWateredOutcome::WateredRecently);

        let returned_plant = get_plant(&conn, &plant1.slug).unwrap().unwrap();

        assert_eq!(returned_plant.last_watered_at, Some(1000));
    }
}
