use std::fs::exists;
use chrono::NaiveDateTime;
use md5::{Digest, Md5};
use rusqlite::{Connection, ToSql, types::FromSql};

use crate::database::{migrations::get_migrations, types::RecentProject};

pub mod migrations;
pub mod types;

pub const DB_PATH: &str = "./appdata.db";
pub const DB_VERSION: u32 = 2;

pub fn get_last_project_path() -> Result<Option<String>, DBError> {
    get_global_setting("last_project_path")
}

pub fn set_last_project_path(path: String) -> Result<(), DBError> {
    update_global_setting("last_project_path", path)
}

pub fn get_last_project_create_path() -> Result<Option<String>, DBError> {
    get_global_setting("last_project_create_path")
}

pub fn set_last_project_create_path(path: String) -> Result<(), DBError> {
    update_global_setting("last_project_create_path", path)
}

pub fn get_last_browse_source_path() -> Result<Option<String>, DBError> {
    get_global_setting("last_browse_source_path")
}

pub fn set_last_browse_source_path(path: String) -> Result<(), DBError> {
    update_global_setting("last_browse_source_path", path)
}

pub fn get_recently_opened(max_count: usize) -> Result<Vec<RecentProject>, DBError> {
    let connection = open_connection()?;
    let mut stmt = connection.prepare("SELECT name, path, last_opened FROM RecentlyOpened ORDER BY last_opened DESC LIMIT ?1").map_err(DBError::SqliteError)?;
    let rows = stmt.query_map([max_count as u32], |r| {
        let name: String = r.get(0)?;
        let path: String = r.get(1)?;
        let last_opened: String = r.get(2)?;
        Ok((name, path, last_opened))
    }).map_err(DBError::SqliteError)?;

    let mut recent = Vec::new();
    for row in rows {
        let Ok((name, path, last_opened)) = row else {
            continue;
        };
        let Ok(time) = NaiveDateTime::parse_from_str(&last_opened, "%F %T") else {
            continue;
        };
        recent.push(RecentProject {
            name,
            path,
            last_opened: time.and_utc()
        });
    }

    Ok(recent)
}

pub fn update_recently_opened(name: String, path: String) -> Result<(), DBError> {
    let mut hasher = Md5::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    let md5 = format!("{:X}", result);

    let connection = open_connection()?;
    connection.execute("INSERT OR REPLACE INTO RecentlyOpened (id, name, path, last_opened) VALUES (?1, ?2, ?3, DATETIME('now'))", [md5, name, path])
        .map_err(DBError::SqliteError)?;

    Ok(())
}

fn get_global_setting<T>(setting_name: &str) -> Result<T, DBError> where T: FromSql {
    let connection = open_connection()?;
    let value = connection.query_one(&format!("SELECT {} FROM GlobalSettings WHERE id = 0", setting_name), (), |r| r.get::<_, T>(0))
        .map_err(DBError::SqliteError)?;

    Ok(value)
}

fn update_global_setting<T>(setting_name: &str, value: T) -> Result<(), DBError> where T: ToSql {
    let connection = open_connection()?;
    connection.execute(&format!("UPDATE GlobalSettings SET {} = ?1", setting_name), [value])
        .map_err(DBError::SqliteError)?;

    Ok(())
}

fn open_connection() -> Result<Connection, DBError> {
    let initialize_db = !exists(DB_PATH).map_err(DBError::IOError)?;
    let mut conn = Connection::open(DB_PATH).map_err(DBError::SqliteError)?;

    // Initialize the database in case it did not exist
    if initialize_db {
        conn.execute_batch(include_str!("schema.sql")).map_err(DBError::SqliteError)?;
    }

    // Check if the database version matches the current application
    let local_version: i64 = conn.query_one("SELECT Version FROM SchemaVersion", (), |r| r.get(0))
        .map_err(DBError::SqliteError)?;

    // TODO: Panic in case there is missing migrations or the db version is newer than the app version

    // Perform any necessary DB version migrations in case of a schema change
    if let Some(migrations) = get_migrations(local_version as u32) {
        for (version, migration) in migrations {
            let tx = conn.transaction().map_err(DBError::SqliteError)?;

            tx.execute_batch(&migration).map_err(DBError::SqliteError)?;
            tx.execute("UPDATE SchemaVersion SET Version = ?1", [version]).map_err(DBError::SqliteError)?;

            tx.commit().map_err(DBError::SqliteError)?;
        }
    };

    Ok(conn)
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DBError {
    SqliteError(rusqlite::Error),
    IOError(std::io::Error)
}
