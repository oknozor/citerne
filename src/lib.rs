//! ## Citerne
//!```toml
//! # Cargo.toml
//! citerne = {version = "0.1", features = ["postgres"] }
//! ```
//!
//! **Embedding diesel migrations:**
//!
//! You must provide a diesel migration directory (relative to your crate root directory).
//! Citerne will set up a database container and execute all migration in the directory.
//!
//! ```rust, ignore
//! #[derive(Insertable)]
//! #[diesel(table_name = dummy)]
//! struct NewDummy { value: String }
//!
//!     #[database_container_test(migrations = ["./migrations"])]
//!     fn simple_migrations(conn: &mut PgConnection) -> CiterneResult<()> {
//!         let value = NewDummy { value: "one".to_string() };
//!
//!         let sql_result = diesel::insert_into(dummy::table)
//!             .values(value)
//!             .execute(conn)?;
//!
//!         assert_eq!(sql_result, 1);
//!
//!         Ok(())
//!     }
//! ```
//! **Custom migration files:**
//!
//! You can directly embed a single sql migration file to populate your containerized database
//! with fixture data.
//!
//! Note that the `migrations` execution is ordered, you always need to provide diesel migrations first.
//!
//! ```rust, ignore
//!     #[database_container_test(migrations = ["./migrations", "./tests/dummy.sql"])]
//!     fn migration_with_data_script(conn: &mut PgConnection) -> CiterneResult<()> {
//!         use crate::schema::dummy::value;
//!         use crate::test::dummy::dsl::dummy;
//!
//!         let dummies: Vec<String> = dummy.select(value).load::<String>(conn)?;
//!
//!         assert_eq!(dummies, vec!["yeah".to_string(), "yo".to_string()]);
//!
//!         Ok(())
//!     }
//! ```
//!
//! **Raw sql script:**
//!
//! Alternatively if you need your fixture data to appear in the test
//! you can use the `sql` attribute. Note that you can combine both approach.
//!
//! ```rust, ignore
//!     #[database_container_test(
//!         migrations = ["./migrations"]
//!         sql = r#"
//!            INSERT INTO dummy (value) VALUES ('yeah');
//!            INSERT INTO dummy (value) VALUES ('yo');
//!         "#
//!     )]
//!     fn migration_with_raw_sql_script(conn: &mut PgConnection) -> CiterneResult<()> {
//!         use crate::schema::dummy::value;
//!         use crate::test::dummy::dsl::dummy;
//!
//!         let dummies: Vec<String> = dummy.select(value).load::<String>(conn)?;
//!
//!         assert_eq!(dummies, vec!["yeah".to_string(), "yo".to_string()]);
//!
//!         Ok(())
//!     }
//! ```
use std::error::Error;

#[cfg(all(feature = "mysql", feature = "postgres"))]
compile_error!("Feature 'mysql' and 'postgres' cannot be enabled together");

#[cfg(test)]
mod schema;

pub use citerne_derive::database_container_test;

pub type CiterneResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

#[cfg(test)]
mod test {
    use crate::schema::dummy;
    use crate::{database_container_test, CiterneResult};
    use diesel::prelude::*;

    #[derive(Insertable)]
    #[diesel(table_name = dummy)]
    struct NewDummy {
        value: String,
    }

    #[derive(Queryable, PartialEq, Eq)]
    #[diesel(table_name = dummy)]
    struct Dummy {
        id: i32,
        value: String,
    }

    #[database_container_test(migrations = ["./migrations"])]
    fn simple_migrations(conn: &mut PgConnection) -> CiterneResult<()> {
        let value = NewDummy {
            value: "one".to_string(),
        };

        let sql_result = diesel::insert_into(dummy::table)
            .values(value)
            .execute(conn)?;

        assert_eq!(sql_result, 1);

        Ok(())
    }

    #[database_container_test(migrations = ["./migrations", "./tests/dummy.sql"])]
    fn migration_with_data_script(conn: &mut PgConnection) -> CiterneResult<()> {
        use crate::schema::dummy::value;
        use crate::test::dummy::dsl::dummy;

        let dummies: Vec<String> = dummy.select(value).load::<String>(conn)?;

        assert_eq!(dummies, vec!["yeah".to_string(), "yo".to_string()]);

        Ok(())
    }

    #[database_container_test(
        migrations = ["./migrations"]
        sql = r#"
           INSERT INTO dummy (value) VALUES ('yeah');
           INSERT INTO dummy (value) VALUES ('yo');
        "#
    )]
    fn migration_with_raw_sql_script(conn: &mut PgConnection) -> CiterneResult<()> {
        use crate::schema::dummy::value;
        use crate::test::dummy::dsl::dummy;

        let dummies: Vec<String> = dummy.select(value).load::<String>(conn)?;

        assert_eq!(dummies, vec!["yeah".to_string(), "yo".to_string()]);

        Ok(())
    }
}
