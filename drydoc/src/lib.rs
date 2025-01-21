#![cfg(not(doctest))]
#![doc = include_str!("../README.md")]

use minijinja::path_loader;
use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitBool, LitStr, Token,
};
use toml::Table;

/// Keep your tests DRY with drydoc!
///
///
/// Accepted arguments:
///
/// - path: The location where to find the template, starting from crate root (where `Cargo.toml` is). (Required) e.g. `path = "docs/mine.rs`.
/// - toml: An inline toml doc containing data for use by JINJA templates if the `jinja` feature (on by default) is enabled. (Optional) e.g. `toml = { ship = "Rocinante" }`
/// - hidden: Set to `false` to prepend each line with a doc comment. This lets you boilerplate that you can hide from your readers. (Optional) e.g. `hidden = true`
///
/// See [crate] docs for usage suggestions.
#[proc_macro]
pub fn drydoc(input: TokenStream) -> TokenStream {
    let Drydoc { contents } = parse_macro_input!(input as Drydoc);

    quote! {
        #contents
    }
    .into()
}

struct Drydoc {
    contents: String,
}

impl Parse for Drydoc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = KeyValueArgs::parse(input)?;
        let cargo_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets this env var"));

        let contents = fs_err::read_to_string(cargo_dir.join(&args.path)).map_err(|error| {
            syn::Error::new(
                input.span(),
                format!(
                    "drydoc: Error reading from `{}`:\n\n{error}",
                    args.path.display()
                ),
            )
        })?;

        let rendered = if cfg!(feature = "jinja") {
            let mut env = minijinja::Environment::new();
            env.set_loader(path_loader(std::env::var("CARGO_MANIFEST_DIR").unwrap()));
            env.add_filter("dochide", dochide_lines);

            #[cfg(feature = "minijinja-contrib")]
            minijinja_contrib::add_to_environment(&mut env);
            let template = env
                .get_template(&format!("{}", args.path.display()))
                .unwrap();
            template.render(args.toml).unwrap()
        } else {
            contents
        };

        if args.hidden {
            Ok(Drydoc {
                contents: dochide_lines(&rendered),
            })
        } else {
            Ok(Drydoc { contents: rendered })
        }
    }
}

#[cfg(feature = "toml")]
fn to_toml_table(input: &str) -> Result<toml::Table, toml::de::Error> {
    // panic!("{input}");
    // panic!("table = {{{}}}", input);
    let table = format!("table = {{{}}}", input).parse::<Table>()?;
    let value = table
        .get("table")
        .expect("internally built toml has this top level key");

    if let toml::Value::Table(table) = value {
        Ok(table.to_owned())
    } else {
        panic!("Expected {value:?} to be a toml::Table but it is not");
    }
}

/// Prepends a doctest hide character `#` in front of each non-hidden line
///
/// Not perfect, assumes each already hidden line has a space after the
/// hash i.e. `# `
fn dochide_lines(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        if !line.trim_start().starts_with("# ") {
            out.push_str("# ");
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// macro!(path = "", toml = { name = "Richard" })
#[derive(Debug)]
struct KeyValueArgs {
    path: PathBuf,
    #[cfg(feature = "toml")]
    toml: toml::Table,
    hidden: bool,
}

#[derive(Debug)]
struct KeyValueArgsBuilder {
    path: Option<PathBuf>,

    #[cfg(feature = "toml")]
    toml: Option<toml::Table>,

    hidden: Option<bool>,
}

impl KeyValueArgsBuilder {
    fn new() -> Self {
        KeyValueArgsBuilder {
            path: None,
            toml: None,
            hidden: None,
        }
    }

    fn path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    #[cfg(feature = "toml")]
    fn toml(&mut self, toml: toml::Table) {
        self.toml = Some(toml);
    }

    fn build(self) -> syn::Result<KeyValueArgs> {
        let path = self
            .path
            .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "missing `path`"))?;
        let hidden = self.hidden.unwrap_or_default();

        #[cfg(feature = "toml")]
        {
            let toml = self.toml.unwrap_or_default();
            Ok(KeyValueArgs { path, toml, hidden })
        }
        #[cfg(not(feature = "toml"))]
        {
            Ok(KeyValueArgs { path, hidden })
        }
    }
}

impl Parse for KeyValueArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut builder = KeyValueArgsBuilder::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "path" => {
                    let path: LitStr = input.parse()?;
                    builder.path(PathBuf::from(path.value()));
                }
                "hidden" => {
                    builder.hidden = Some(input.parse::<LitBool>()?.value());
                }
                "toml" => {
                    if cfg!(feature = "toml") {
                        let content;
                        braced!(content in input);
                        let tokens: proc_macro2::TokenStream = content.parse()?;
                        builder.toml(to_toml_table(&tokens.to_string()).unwrap());
                    } else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "toml values provided but `jinja` feature not enabled",
                        ));
                    }
                }
                _ => return Err(syn::Error::new(ident.span(), "unexpected identifier")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kwarg_style_args_toml_nested() {
        let input =
            r#"path = "path/to/doc.rs", toml = { number = 15 , person = { name = "richard" } }"#;
        let args: KeyValueArgs = syn::parse_str(input).unwrap();

        assert_eq!(args.path, PathBuf::from("path/to/doc.rs"));
        assert_eq!(
            to_toml_table(r#"number = 15 , person = { name = "richard" }"#).unwrap(),
            args.toml,
        );
    }

    #[test]
    fn test_parse_kwarg_style_args() {
        let input = r#"path = "path/to/doc.rs", toml = { number = 15 }"#;
        let args: KeyValueArgs = syn::parse_str(input).unwrap();

        assert_eq!(PathBuf::from("path/to/doc.rs"), args.path);
        assert_eq!(to_toml_table(r#"number = 15"#).unwrap(), args.toml);
    }

    #[test]
    fn test_parse_bad_key_err() {
        let input = r#"path = "path/to/doc.rs", yolo: {}""#;
        let result: syn::Result<KeyValueArgs> = syn::parse_str(input);
        assert!(result.is_err(), "Not err {result:?}");
    }

    #[test]
    fn test_un_hidden_lines() {
        assert_eq!("# use std::fs;\n", &dochide_lines("use std::fs;\n"));

        assert_eq!("# use std::fs;\n", &dochide_lines("# use std::fs;\n"));
        assert_eq!("  # use std::fs;\n", &dochide_lines("  # use std::fs;\n"));

        assert_eq!("# #[derive(Debug)]\n", &dochide_lines("#[derive(Debug)]"));
        assert_eq!("# #[derive(Debug)]\n", &dochide_lines("# #[derive(Debug)]"));
    }
}
