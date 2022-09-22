use proc_macro::TokenStream;

use crate::attributes::{DieselTankerAttributes};
use quote::{quote};
use syn::{LitStr, parse_macro_input, parse_quote, Stmt};

mod attributes;

#[proc_macro_attribute]
pub fn database_container(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as syn::ItemFn);
    let attributes = parse_macro_input!(attr as DieselTankerAttributes);

    let mut init_container_statements: Vec<Stmt> = parse_quote! {
            let docker = testcontainers::clients::Cli::default();
            let postgres: testcontainers::Container<testcontainers::images::postgres::Postgres> = docker.run(testcontainers::images::postgres::Postgres::default());
            let addr = format!(
                "postgresql://postgres@127.0.0.1:{}/postgres",
                postgres.get_host_port_ipv4(5432)
            );

            let mut conn = {
                use diesel::Connection;
                diesel::PgConnection::establish(&addr)?
            };
        };


    for migration in attributes.migrations {
        let run_migrations = parse_quote! {
            let crate_path = std::env::var("CARGO_MANIFEST_DIR").map(|dir|std::path::PathBuf::from(dir))?;
            let migration_path = crate_path.join(#migration);
            if !migration_path.is_dir() {
                let sql = std::fs::read_to_string(migration_path)?;
                {
                    use diesel::connection::SimpleConnection;
                    conn.batch_execute(&sql)?;
                }
            } else {
                let migrations = diesel_migrations::FileBasedMigrations::from_path(migration_path)?.migrations()?;
                for migration in migrations {
                    migration.run(&mut conn)?;
                }
            }
        };

        init_container_statements.extend::<Vec<Stmt>>(run_migrations);
    }

    init_container_statements.extend(function.block.stmts.clone());
    function.block.stmts = init_container_statements;

    let function = quote! {
        #function
    };

    TokenStream::from(function)
}