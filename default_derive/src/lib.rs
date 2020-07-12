extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, TokenStream, TokenTree};

use std::iter::Peekable;

#[derive(Debug)]
enum VisibilityModifier {
    Pub,
    PubCrate,
}

#[derive(Debug)]
enum DataType {
    Enum,
    Struct(Struct),
}

#[derive(Debug)]
struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
struct Field {
    pub visibility: Option<VisibilityModifier>,
    pub name: Ident,
    pub typ: Ident,
}

/// Parses a visibility modifier such as `pub`. This function will panic if it encounters
/// a visibility modifier with an additional group specifier like `pub(crate)`
fn next_visibility_modifier(
    input: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Option<VisibilityModifier> {
    if let Some(TokenTree::Ident(ident)) = input.peek() {
        if ident.to_string() == "pub" {
            // Consume this token
            input.next();

            // TODO: We do not handle any modifiers besides `pub`
            if let Some(TokenTree::Group(_)) = input.peek() {
                panic!("Visibility modifies like pub(crate) are not yet handled");
            }

            return Some(VisibilityModifier::Pub);
        }
    }

    None
}

/// Parses a data type such as `struct` or `enum`.
fn next_data_type<'a>(input: &'a mut Peekable<impl Iterator<Item = TokenTree>>) -> DataType {
    // The next token should either be a `struct` or `enum`
    if let Some(TokenTree::Ident(ident)) = input.peek() {
        if ident.to_string() != "struct" {
            panic!("Data type `{}` is not yet supported", ident);
        }

        // Consume the data type identifier
        input.next();

        let struct_name = next_ident(input).unwrap_or_else(|| {
            let next = input.peek();
            panic!("Expected a struct name, got {:?}", next);
        });

        // TODO: Handle unit struct (`struct Foo;`)
        let group = next_group(input)
            .unwrap_or_else(|| panic!("Expected a struct group, got {:?}", input.peek()));

        // Examine the group delimiter to determine if this is a named or unnamed struct
        match group.delimiter() {
            Delimiter::Brace => {}
            delim => panic!("Unsupported group delimiter: {:?}. Currently only braces (named structs) are supported", delim)
        }

        let mut group_body = group.stream().into_iter().peekable();

        return DataType::Struct(Struct {
            name: struct_name,
            fields: next_fields(&mut group_body),
        });
    }

    panic!("Unexpected token: {:?}", input.next());
}

fn next_fields<'a>(input: &'a mut Peekable<impl Iterator<Item = TokenTree>>) -> Vec<Field> {
    let mut fields = Vec::new();

    // Loop until we reach EOF of this group we're in
    while !next_eof(input) {
        // At the beginning of this loop we should expect one of the following:
        // `pub name: Type`
        // `name: Type`

        // Try to parse a visibility modifier
        let visibility_modifier = next_visibility_modifier(input);

        // Parse out the field name
        let name = next_ident(input)
            .unwrap_or_else(|| panic!("failed to parse field name (got: {:?})", input.peek()));

        // Parse the literal colon `:`
        next_matching_punct(input, ":").unwrap_or_else(|| {
            panic!(
                "expected a colon following a struct field name, got: {:?}",
                input.peek()
            )
        });

        // Parse the field type.
        // TODO: This does not handle generics, lifetimes, references, or `fully::qualified::path::Names`.
        let typ = next_ident(input).expect("failed to parse field type");

        // Finally, parse out trailing commas if they exist
        next_matching_punct(input, ",");

        fields.push(Field {
            visibility: visibility_modifier,
            name,
            typ,
        });
    }

    fields
}

/// Consumes and returns the next `[proc_macro::Ident]`. Panics if the next token is
/// empty or not an `Ident`.
fn next_ident(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<Ident> {
    if let Some(TokenTree::Ident(_)) = input.peek() {
        // consume this token
        if let Some(TokenTree::Ident(token)) = input.next() {
            return Some(token);
        }
    }

    None
}

/// Consumes and returns the next `[proc_macro::Group]`. This is something like: `(crate)`, `{stuff_in_braces}`,
/// `[stuff_in_brackets]`.
fn next_group<'a>(input: &'a mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<Group> {
    if let Some(TokenTree::Group(_)) = input.peek() {
        // consume this token
        if let Some(TokenTree::Group(group)) = input.next() {
            return Some(group);
        }
    }

    None
}

/// Returns whether the `input` has reached end-of-file
fn next_eof(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> bool {
    input.peek().is_none()
}

/// Consumes and returns the next `[proc_macro::Punct]` if it matches the given string.
fn next_matching_punct<'a>(
    input: &'a mut Peekable<impl Iterator<Item = TokenTree>>,
    matching_str: &str,
) -> Option<Punct> {
    if let Some(TokenTree::Punct(punct)) = input.peek() {
        if punct.to_string() != matching_str {
            return None;
        }

        // Consume this token
        if let Some(TokenTree::Punct(punct)) = input.next() {
            return Some(punct);
        }
    }

    None
}

#[proc_macro_derive(OurDefault)]
pub fn default_derive(input: TokenStream) -> TokenStream {
    // We assume an input that looks loosely like the following:
    //
    // pub struct Foo {
    //    pub a: String,
    //    b: usize,
    // }
    //
    // At this times enums, lifetime specifiers, unnamed fields, and generic parameters
    // are not supported.

    // Convert the input to an iterator so we can start peeking tokens
    let mut source = input.into_iter().peekable();

    // We assume that the first token is going to be either a keyword like `pub`
    // or `pub(crate), or the data type.
    let _visibility = next_visibility_modifier(&mut source);

    // Parse the struct
    let parsed_data_type = next_data_type(&mut source);

    let result_text = match parsed_data_type {
        DataType::Struct(s) => {
            // Map each field to the form of:
            // `field_name: Default::default(),`
            let field_initializers = s
                .fields
                .iter()
                .map(|field| format!("{}: Default::default()", field.name))
                .collect::<Vec<String>>()
                .join("\n,");

            // Struct has been parsed -- let's emit our impl
            format!(
                "#[automatically_derived] \
                impl crate::OurDefault for {} {{ \
                    fn our_default() -> Self {{ \
                        {} {{
                            {}
                        }}
                    }}
                }}",
                s.name, s.name, field_initializers
            )
        }
        _ => panic!("Only structs are currently supported"),
    };

    result_text
        .parse::<TokenStream>()
        .expect("proc macro generated invalid tokens")
}
