use std::error::Error;
use std::path::PathBuf;
use diesel::migration::{Migration, MigrationSource};
use diesel_migrations::{embed_migrations, EmbeddedMigration, FileBasedMigrations, MigrationHarness};
use gazole_test_derive::database_container;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;




#[test]
#[database_container(migrations = ["./migrations", "./tests/dummy.sql"])]
fn dummy_container_test() -> Result<()> {
    // Container should setup and teardown
    Ok(())
}
