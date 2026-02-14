CREATE TABLE IF NOT EXISTS GlobalSettings (
    id BOOLEAN PRIMARY KEY,
    last_project_path TEXT,
    last_project_create_path TEXT,
    last_browse_source_path TEXT
);

CREATE TABLE IF NOT EXISTS RecentlyOpened (
    id CHARACTER(32) PRIMARY KEY,
    path TEXT NOT NULL,
    name VARCHAR(255) NOT NULL,
    last_opened DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS SchemaVersion (
    version INTEGER NOT NULL PRIMARY KEY
);

INSERT INTO SchemaVersion (version) VALUES (2);
INSERT INTO GlobalSettings (id) VALUES (0);
