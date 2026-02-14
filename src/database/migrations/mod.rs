use std::sync::LazyLock;

use crate::database::DB_VERSION;

// Migrations for DB / schema changes. The list has to be sorted by DB version.
// The version number is the version this migration migrates to (e.g. a key of 2
// would migrate a database with schema version 1 to schema version 2)
static MIGRATIONS: LazyLock<Vec<(u32, LazyLock<&str>)>> = LazyLock::new(|| vec![
    (2, LazyLock::new(|| include_str!("v2_add_more_path_caches.sql")))
]);

/// Gets all migration SQLs required for the database with the given schema version
/// to be updated to the newest version. If already on the newest version, the return value
/// will be [`None`].
pub fn get_migrations(current_version: u32) -> Option<Vec<(u32, String)>> {
    let version_diff = (DB_VERSION - current_version) as usize;
    if version_diff == 0 {
        return None;
    }

    // Collect all migrations
    let mut migrations = Vec::with_capacity(version_diff);
    for (version, migration) in MIGRATIONS.iter().rev() {
        if *version <= current_version {
            break;
        }
        migrations.insert(0, (*version, migration.to_string()));
    }

    Some(migrations)
}
