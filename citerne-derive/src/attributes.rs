use syn::parse::{Parse, ParseStream};
use syn::{bracketed, LitStr, Token};

pub struct CiterneAttributes {
    pub migrations: Vec<String>,
    pub sql: Option<String>,
}

impl Parse for CiterneAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = CiterneAttributes {
            migrations: vec![],
            sql: None,
        };

        while let Ok(ident) = input.parse::<syn::Ident>() {
            input.parse::<Token!(=)>()?;
            match ident.to_string().as_str() {
                "migrations" => attributes.migrations = Self::parse_migrations(input)?,
                "sql" => attributes.sql = Some(input.parse::<LitStr>()?.value()),
                other => panic!(
                    "unexpected attribute {}, supported attributes are : ['migrations', 'sql']",
                    other
                ),
            }

            if input.peek(Token!(,)) {
                input.parse::<Token!(,)>()?;
            }
        }

        if attributes.migrations.is_empty() {
            panic!(
                r#"
proc_macro 'database_container_test' expect at least one migration.
example : `#[database_container_test(migrations = ["./migrations"])]`
"#
            );
        }

        Ok(attributes)
    }
}

impl CiterneAttributes {
    fn parse_migrations(input: ParseStream) -> syn::Result<Vec<String>> {
        let content;
        bracketed!(content in input);
        let mut migrations = vec![];
        while let Ok(migration) = content.parse::<LitStr>() {
            migrations.push(migration.value());
            if !content.is_empty() {
                content.parse::<Token!(,)>()?;
            }
        }

        Ok(migrations)
    }
}
