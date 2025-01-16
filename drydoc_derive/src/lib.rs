extern crate proc_macro;
use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

fn contents_from_input_path(input: LitStr) -> syn::Result<String> {
    let cargo_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets this env var"));

    let path = input.value();
    fs_err::read_to_string(cargo_dir.join(&path)).map_err(|error| {
        syn::Error::new(
            input.span(),
            format!("drydoc: Error reading from `{path}`:\n\n{error}"),
        )
    })
}

/// The same thing as `include_str!` but it's relative from the crate root
#[proc_macro]
pub fn doc_show(input: TokenStream) -> TokenStream {
    contents_from_input_path(parse_macro_input!(input as LitStr))
        .map(|contents| quote! { #contents })
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Reads a file relative from crate root and appends a doc hide token `#` to each line
#[proc_macro]
pub fn doc_hide(input: TokenStream) -> TokenStream {
    contents_from_input_path(parse_macro_input!(input as LitStr))
        .map(|contents| dochide_lines(&contents))
        .map(|hidden| quote! {#hidden})
        .unwrap_or_else(syn::Error::into_compile_error)
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
    fn test_un_hidden_lines() {
        assert_eq!("# use std::fs;\n", &dochide_lines("use std::fs;\n"));

        assert_eq!("# use std::fs;\n", &dochide_lines("# use std::fs;\n"));
        assert_eq!("  # use std::fs;\n", &dochide_lines("  # use std::fs;\n"));

        assert_eq!("# #[derive(Debug)]\n", &dochide_lines("#[derive(Debug)]"));
        assert_eq!("# #[derive(Debug)]\n", &dochide_lines("# #[derive(Debug)]"));
    }
}
