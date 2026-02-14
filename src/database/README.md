# Database
App settings and other relevant data (e.g. recently opened projects, etc.) are stored inside a local SQLite database called `appdata.db`. The database is located in the same directory as the executable. It will be created if it does not exist yet. The database is versioned and in case of changes (e.g. schema changes), it will automatically be migrated to the newer version by the application. Rolling back to a previous version is not supported.

## SQLs
The `schema.sql` file in this directory contains the SQLs to initialize a new database with the most current database version. All SQL files found in the `migrations` sub-directory are used to migrate an older database version to a newer one.

## Migrations
Migrations are located in the `migrations` sub-directory. They have a naming convention of `v[VERSION]_[CHANGE_SUMMARY].sql`. The `[VERSION]` refers to the version this migration migrates to. Therefore e.g. a database with version 1 can be migrated to version 2 by executing the `v2_xxx.sql` SQL-file. Larger version jumps require to run all migrations until the newest version. E.g. a database with version 1 can be migrated to version 4 by executing the `v2_xxx.sql`, `v3_xxx.sql` and `v4_xxx.sql` SQL-files in that exact order.
