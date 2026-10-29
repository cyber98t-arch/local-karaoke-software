use rusqlite::{Connection, TransactionBehavior};
use std::path::Path;

const CURRENT_SCHEMA_VERSION: u32 = 2;

pub struct Catalog {
    connection: Connection,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CatalogSource {
    pub source_path: String,
    pub content_sha256: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration_ms: u64,
}

impl Catalog {
    pub fn open(database_path: &Path) -> rusqlite::Result<Self> {
        let mut connection = Connection::open(database_path)?;
        connection.pragma_update(None, "foreign_keys", true)?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> rusqlite::Result<u32> {
        self.connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
    }

    pub fn replace_sources(&mut self, sources: &[CatalogSource]) -> rusqlite::Result<()> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute("DELETE FROM library_sources", [])?;
        {
            let mut insert = transaction.prepare(
                "INSERT INTO library_sources (
                    source_path, content_sha256, title, artist, duration_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for source in sources {
                let duration_ms = source.duration_ms.min(i64::MAX as u64) as i64;
                insert.execute(rusqlite::params![
                    source.source_path,
                    source.content_sha256,
                    source.title,
                    source.artist,
                    duration_ms,
                ])?;
            }
        }
        transaction.commit()
    }

    pub fn sources(&self) -> rusqlite::Result<Vec<CatalogSource>> {
        let mut query = self.connection.prepare(
            "SELECT source_path, content_sha256, title, artist, duration_ms
             FROM library_sources
             ORDER BY source_path",
        )?;
        let sources = query
            .query_map([], |row| {
                let duration_ms = row.get::<_, i64>(4)?;
                Ok(CatalogSource {
                    source_path: row.get(0)?,
                    content_sha256: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    duration_ms: duration_ms.max(0) as u64,
                })
            })?
            .collect();
        sources
    }
}

fn migrate(connection: &mut Connection) -> rusqlite::Result<()> {
    let schema_version =
        connection.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))?;

    match schema_version {
        0 => {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE library_sources (
                    id INTEGER PRIMARY KEY,
                    source_path TEXT NOT NULL UNIQUE,
                    content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64),
                    title TEXT,
                    artist TEXT,
                    duration_ms INTEGER NOT NULL DEFAULT 0 CHECK (duration_ms >= 0)
                );",
            )?;
            transaction.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)?;
            transaction.commit()?;
        }
        1 => {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "ALTER TABLE library_sources ADD COLUMN title TEXT;
                 ALTER TABLE library_sources ADD COLUMN artist TEXT;
                 ALTER TABLE library_sources ADD COLUMN duration_ms INTEGER NOT NULL DEFAULT 0 CHECK (duration_ms >= 0);",
            )?;
            transaction.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)?;
            transaction.commit()?;
        }
        CURRENT_SCHEMA_VERSION => {}
        _ => return Err(rusqlite::Error::InvalidQuery),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Catalog;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("hibiki-catalog-{}-{unique}", std::process::id()));
            fs::create_dir(&path).expect("test application-data directory should be created");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path)
                .expect("test application-data directory should be removable");
        }
    }

    #[test]
    fn creates_current_schema_at_injected_application_data_path() {
        let directory = TestDirectory::new();
        let database_path = directory.path.join("catalog.sqlite3");

        let catalog = Catalog::open(&database_path).expect("catalog should open");

        assert!(database_path.is_file());
        assert_eq!(
            catalog
                .schema_version()
                .expect("schema version should be readable"),
            2
        );
        assert_eq!(
            catalog
                .connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = 'library_sources'",
                    [],
                    |row| row.get::<_, u32>(0),
                )
                .expect("schema should be queryable"),
            1
        );
    }

    #[test]
    fn reopening_catalog_preserves_existing_records() {
        let directory = TestDirectory::new();
        let database_path = directory.path.join("catalog.sqlite3");
        {
            let catalog = Catalog::open(&database_path).expect("catalog should open");
            catalog
                .connection
                .execute(
                    "INSERT INTO library_sources (source_path, content_sha256) VALUES (?1, ?2)",
                    ["C:\\Music\\song.flac", &"a".repeat(64)],
                )
                .expect("fixture record should be inserted");
        }

        let reopened = Catalog::open(&database_path).expect("catalog should reopen");

        assert_eq!(
            reopened
                .connection
                .query_row("SELECT COUNT(*) FROM library_sources", [], |row| {
                    row.get::<_, u32>(0)
                })
                .expect("record count should be readable"),
            1
        );
    }

    #[test]
    fn migrates_version_one_without_losing_existing_records() {
        let directory = TestDirectory::new();
        let database_path = directory.path.join("catalog.sqlite3");
        {
            let connection = rusqlite::Connection::open(&database_path)
                .expect("version one fixture should open");
            connection
                .execute_batch(
                    "CREATE TABLE library_sources (
                        id INTEGER PRIMARY KEY,
                        source_path TEXT NOT NULL UNIQUE,
                        content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64)
                    );
                    PRAGMA user_version = 1;",
                )
                .expect("version one schema should be created");
            connection
                .execute(
                    "INSERT INTO library_sources (source_path, content_sha256) VALUES (?1, ?2)",
                    ["C:\\Music\\legacy.flac", &"b".repeat(64)],
                )
                .expect("legacy record should be inserted");
        }

        let catalog = Catalog::open(&database_path).expect("catalog should migrate");

        assert_eq!(
            catalog
                .schema_version()
                .expect("version should be readable"),
            2
        );
        let sources = catalog
            .sources()
            .expect("migrated records should be readable");
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_path, "C:\\Music\\legacy.flac");
    }
}
