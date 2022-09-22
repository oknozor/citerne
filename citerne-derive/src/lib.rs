extern crate core;

use proc_macro::TokenStream;

use crate::attributes::CiterneAttributes;
use quote::quote;
use syn::{parse_macro_input, parse_quote, FnArg, Ident, Pat, Stmt};

mod attributes;

const PG_SIG_ERROR_MESSAGE: &str =
    "database_container_test should have `fn(conn: &mut PgConnection)` signature";

#[proc_macro_attribute]
pub fn database_container_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as syn::ItemFn);

    let arg = function
        .sig
        .inputs
        .pop()
        .unwrap_or_else(|| panic!("{PG_SIG_ERROR_MESSAGE}"));

    let ident = match arg.value() {
        FnArg::Receiver(_) => panic!("{PG_SIG_ERROR_MESSAGE}"),
        FnArg::Typed(typed) => {
            if typed.ty != syn::parse_quote!(&mut PgConnection) {
                panic!("{PG_SIG_ERROR_MESSAGE}");
            }

            if let Pat::Ident(ident) = typed.pat.as_ref() {
                ident.ident.clone()
            } else {
                panic!("{PG_SIG_ERROR_MESSAGE}");
            }
        }
    };

    let attributes = parse_macro_input!(attr as CiterneAttributes);

    #[cfg(feature = "postgres")]
    let mut init_container_statements = generate_postgres_container_init(attributes, ident);

    #[cfg(feature = "mysql")]
    let mut init_container_statements = generate_mysql_container_init(attributes);

    init_container_statements.extend(function.block.stmts.clone());
    function.block.stmts = init_container_statements;

    let function = quote! {
        #[test]
        #function
    };

    TokenStream::from(function)
}

#[cfg(feature = "postgres")]
fn generate_postgres_container_init(attributes: CiterneAttributes, conn_ident: Ident) -> Vec<Stmt> {
    let mut init_container_statements: Vec<Stmt> = parse_quote! {
        let docker = testcontainers::clients::Cli::default();
        println!("Starting postgresql container");
        let __postgres_container__: testcontainers::Container<testcontainers::images::postgres::Postgres> = docker.run(testcontainers::images::postgres::Postgres::default());
        let __postgres_addr__ = format!(
            "postgresql://postgres@127.0.0.1:{}/postgres",
            __postgres_container__.get_host_port_ipv4(5432)
        );
        println!("Container started, trying to establish postgresql connection on port 5432");
        let mut #conn_ident = {
            use diesel::Connection;
            diesel::PgConnection::establish(&__postgres_addr__)?
        };
        let #conn_ident = &mut #conn_ident;
    };

    for migration in attributes.migrations {
        let mut run_migrations: Vec<Stmt> = parse_quote! {
            let crate_path = std::env::var("CARGO_MANIFEST_DIR").map(|dir|std::path::PathBuf::from(dir))?;
            let migration_path = crate_path.join(#migration);
            if !migration_path.is_dir() {
                let sql = std::fs::read_to_string(&migration_path)?;
                {
                    use diesel::connection::SimpleConnection;
                    println!("Running single migration {}", migration_path.file_name().unwrap().to_string_lossy());
                    #conn_ident.batch_execute(&sql)?;
                }
            } else {
                use diesel::migration::MigrationSource;
                let migrations = diesel_migrations::FileBasedMigrations::from_path(&migration_path)?.migrations()?;
                for migration in migrations {
                    println!("Running migrations {}", migration.name());
                    migration.run(conn)?;
                }
            }
        };

        if let Some(sql) = attributes.sql.clone() {
            run_migrations.push(parse_quote! {
                {
                    use diesel::connection::SimpleConnection;
                    #conn_ident.batch_execute(#sql)?;
                }
            });
        }

        init_container_statements.extend::<Vec<Stmt>>(run_migrations);
    }
    init_container_statements
}