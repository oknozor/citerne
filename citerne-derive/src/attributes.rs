use syn::parse::{Parse, ParseStream};
use syn::{bracketed, LitStr, Token};

pub struct DieselTankerAttributes {
    pub migrations: Vec<String>,
}

impl Parse for DieselTankerAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = DieselTankerAttributes {
            migrations: vec![]
        };

        while let Ok(ident) = input.parse::<syn::Ident>() {
            input.parse::<Token!(=)>()?;
            match ident.to_string().as_str() {
                "migrations" => attributes.migrations = Self::parse_migrations(input)?,
                other => panic!(
                    "unexpected attribute {}, supported attributes are : ['migrations']",
                    other
                ),
            }

            if input.peek(Token!(,)) {
                input.parse::<Token!(,)>()?;
            }
        }

        Ok(attributes)
    }
}

impl DieselTankerAttributes {
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
