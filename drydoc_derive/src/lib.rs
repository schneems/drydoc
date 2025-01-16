use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitStr, Token,
};

/// "path/to/doc.rs", context: { number => 15 }
#[derive(Debug, PartialEq, Eq)]
struct DrydocArgs {
    path: PathBuf,
    context: Option<String>,
}

impl Parse for DrydocArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the file path
        let path: LitStr = input.parse()?;
        let path = PathBuf::from(path.value());

        let _ = input.parse::<Token![,]>().ok();
        let context = if input.peek(Ident) && input.peek2(Token![:]) {
            let ident: Ident = input.parse()?;

            if ident == "context" {
                input.parse::<Token![:]>()?;
                let content;
                braced!(content in input);

                // Consume the parse buffer and convert it into a string
                let tokens: proc_macro2::TokenStream = content.parse()?;
                Some(tokens.to_string())
            } else {
                return Err(syn::Error::new(ident.span(), "expected `context`"));
            }
        } else {
            None
        };

        let args = DrydocArgs { path, context };
        Ok(args)
    }
}

struct Drydoc {
    #[allow(dead_code)]
    args: DrydocArgs,
    contents: String,
}

impl Parse for Drydoc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = DrydocArgs::parse(input)?;
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
        Ok(Drydoc { args, contents })
    }
}

/// The same thing as `include_str!` but it's relative from the crate root
#[proc_macro]
pub fn doc_show(input: TokenStream) -> TokenStream {
    let Drydoc { args: _, contents } = parse_macro_input!(input as Drydoc);

    quote! {
        #contents
    }
    .into()
}

/// Reads a file relative from crate root and appends a doc hide token `#` to each line
#[proc_macro]
pub fn doc_hide(input: TokenStream) -> TokenStream {
    let Drydoc { args: _, contents } = parse_macro_input!(input as Drydoc);

    let hidden = dochide_lines(&contents);
    quote! {
        #hidden
    }
    .into()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let input = r#""path/to/doc.rs", context: { number => 15 }"#;
        let args: DrydocArgs = syn::parse_str(input).unwrap();

        assert_eq!(args.path, PathBuf::from("path/to/doc.rs"));
        assert_eq!(args.context, Some(r#"number => 15"#.to_string()));
    }

    #[test]
    fn test_parse_args_no_context() {
        let input = r#""path/to/doc.rs""#;
        let args: DrydocArgs = syn::parse_str(input).expect("Failed to parse Args");
        assert_eq!(args.path, PathBuf::from("path/to/doc.rs"));
        assert_eq!(args.context, None);
    }

    #[test]
    fn test_parse_bad_key_err() {
        let input = r#""path/to/doc.rs", yolo: {}""#;
        let result: syn::Result<DrydocArgs> = syn::parse_str(input);
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
