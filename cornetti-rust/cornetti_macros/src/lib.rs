use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

#[proc_macro]
pub fn define_errors(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    let groups = parse_error_groups(input);

    let mut mod_tokens = TokenStream2::new();
    let mut catalog_entries = TokenStream2::new();

    for group in &groups {
        let group_name = format_ident!("{}", group.name);
        let group_attrs = &group.attrs;

        let mut factory_tokens = TokenStream2::new();
        for entry in &group.entries {
            let fn_name = format_ident!("{}", entry.name);
            let status = entry.status.unwrap_or(group.status);
            let log_level = entry.log_level.clone().or(group.log_level.clone());
            let log_level_tokens = match log_level {
                Some(l) => {
                    let lvl = format_ident!("{}", l.to_uppercase());
                    quote! { Some(tracing::Level::#lvl) }
                }
                None => quote! { None },
            };
            let detail = &entry.detail;
            let corr_id = format!("BE_{}", entry.name.to_uppercase());

            if entry.has_param {
                factory_tokens.extend(quote! {
                    pub fn #fn_name() -> crate::core::models::CornettiError {
                        crate::core::models::CornettiError {
                            status: crate::core::http_status::HttpStatus::from(#status),
                            detail: #detail.into(),
                            corr_id: #corr_id.into(),
                            log_level: #log_level_tokens,
                            internal_detail: String::new(),
                        }
                    }
                });
            } else {
                factory_tokens.extend(quote! {
                    pub fn #fn_name() -> crate::core::models::CornettiError {
                        crate::core::models::CornettiError {
                            status: crate::core::http_status::HttpStatus::from(#status),
                            detail: #detail.into(),
                            corr_id: #corr_id.into(),
                            log_level: #log_level_tokens,
                            internal_detail: #detail.into(),
                        }
                    }
                });
            }

            catalog_entries.extend(quote! {
                c.push(#group_name::#fn_name());
            });
        }

        mod_tokens.extend(quote! {
            #(#group_attrs)*
            pub mod #group_name {
                #factory_tokens
            }
        });
    }

    let catalog_fn = quote! {
        pub fn error_catalog() -> Vec<crate::core::models::CornettiError> {
            let mut c = Vec::new();
            #catalog_entries
            c
        }
    };

    let expanded = quote! {
        #mod_tokens
        #catalog_fn
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn export_errors_json(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    let groups = parse_error_groups(input);

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let json = generate_errors_json(&groups);
        let path = format!("{}/server-errors.json", manifest_dir);
        let _ = std::fs::write(&path, json);
    }

    TokenStream::from(quote! {})
}

struct Group {
    attrs: Vec<TokenStream2>,
    name: String,
    status: u16,
    log_level: Option<String>,
    entries: Vec<Entry>,
}

struct Entry {
    name: String,
    status: Option<u16>,
    log_level: Option<String>,
    detail: String,
    has_param: bool,
}

fn generate_errors_json(groups: &[Group]) -> String {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    for group in groups {
        for entry in &group.entries {
            let corr_id = format!("BE_{}", entry.name.to_uppercase());
            map.insert(corr_id, entry.detail.clone());
        }
    }
    serde_json::to_string_pretty(&map).unwrap_or_else(|_| "{}".into())
}

fn parse_error_groups(input: TokenStream2) -> Vec<Group> {
    let mut groups = Vec::new();
    let mut tokens: Vec<TokenTree> = input.into_iter().collect();

    // Pre-expand include!("path") tokens — proc macros receive them as-is.
    let mut i = 0;
    while i + 2 < tokens.len() {
        match (&tokens[i], &tokens[i + 1], &tokens[i + 2]) {
            (TokenTree::Ident(id), TokenTree::Punct(p), TokenTree::Group(g))
                if *id == "include" && p.as_char() == '!' =>
            {
                let inner = g.stream().to_string();
                let rel_path = inner.trim().trim_matches('"').to_string();
                let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let abs_path = std::path::Path::new(&manifest).join(&rel_path);
                let file_content = std::fs::read_to_string(&abs_path)
                    .unwrap_or_else(|_| panic!("failed to read include file: {}", abs_path.display()));
                let file_tokens: TokenStream2 = file_content
                    .parse()
                    .unwrap_or_else(|_| panic!("failed to parse include file: {}", abs_path.display()));
                let file_tokens: Vec<TokenTree> = file_tokens.into_iter().collect();
                let added = file_tokens.len();
                tokens.splice(i..i + 3, file_tokens);
                i += added;
            }
            _ => i += 1,
        }
    }

    let mut tokens = tokens.into_iter().peekable();

    while tokens.peek().is_some() {
        // Parse optional #[cfg(...)] attributes
        let mut attrs = Vec::new();
        while let Some(tt) = tokens.peek() {
            let s = tt.to_string();
            if s.as_str() == "#" {
                tokens.next();
                let next = tokens.next().expect("expected '[' after #");
                let next_s = next.to_string();
                if next_s.as_str() == "[" {
                    let attr_tokens = vec![
                        TokenTree::from(proc_macro2::Punct::new('#', proc_macro2::Spacing::Joint)),
                        TokenTree::from(proc_macro2::Group::new(
                            proc_macro2::Delimiter::Bracket,
                            collect_until(&mut tokens, proc_macro2::Delimiter::Bracket),
                        )),
                    ];
                    // Reconstruct as TokenStream2
                    let attr_ts: TokenStream2 = attr_tokens.into_iter().collect();
                    attrs.push(attr_ts);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Parse group name
        let name = expect_ident(&mut tokens, "nome gruppo");
        let name_str = name.to_string();

        let (status, log_level) = expect_group_parens(&mut tokens);

        // Expect ':'
        expect_punct(&mut tokens, ':');

        // Parse entries
        let entries = parse_entries(&mut tokens);

        groups.push(Group {
            attrs,
            name: name_str,
            status,
            log_level,
            entries,
        });

        // Eat optional trailing ','
        if let Some(tt) = tokens.peek()
            && tt.to_string().as_str() == ","
        {
            tokens.next();
        }
    }

    groups
}

fn expect_ident(tokens: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>, ctx: &str) -> proc_macro2::Ident {
    match tokens.next() {
        Some(TokenTree::Ident(id)) => id,
        Some(tt) => panic!("expected identifier in {}, got: {}", ctx, tt),
        None => panic!("unexpected end of input in {}", ctx),
    }
}

fn expect_punct(tokens: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>, ch: char) {
    match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == ch => {}
        Some(tt) => panic!("expected '{}', got: {}", ch, tt),
        None => panic!("unexpected end of input, expected '{}'", ch),
    }
}

fn expect_group_parens(tokens: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>) -> (u16, Option<String>) {
    let tt = tokens.next().expect("expected group parentheses");
    let group = match tt {
        TokenTree::Group(g) if g.delimiter() == proc_macro2::Delimiter::Parenthesis => g,
        _ => panic!("expected (...), got: {}", tt),
    };

    let inner_tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut inner = inner_tokens.into_iter().peekable();

    // Parse status (u16)
    let status = parse_u16(&mut inner);

    // Parse optional log_level
    let log_level = if inner.peek().is_some() {
        expect_punct(&mut inner, ',');
        // expect "log_level"
        let kw = expect_ident(&mut inner, "log_level keyword").to_string();
        if kw.as_str() == "log_level" {
            expect_punct(&mut inner, ':');
            Some(expect_ident(&mut inner, "log_level value").to_string())
        } else {
            panic!("expected 'log_level', got: {}", kw);
        }
    } else {
        None
    };

    (status, log_level)
}

fn parse_entries(tokens: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>) -> Vec<Entry> {
    // Expect {
    let tt = tokens.next().expect("expected '{'");
    let group = match tt {
        TokenTree::Group(g) if g.delimiter() == proc_macro2::Delimiter::Brace => g,
        _ => panic!("expected {{ ... }}, got: {}", tt),
    };

    let inner_tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut inner = inner_tokens.into_iter().peekable();
    let mut entries = Vec::new();

    while inner.peek().is_some() {
        // Check for * prefix (has_param)
        let has_param = if let Some(TokenTree::Punct(p)) = inner.peek() {
            if p.as_char() == '*' {
                inner.next();
                true
            } else {
                false
            }
        } else {
            false
        };

        let name = expect_ident(&mut inner, "nome entry").to_string();

        // Optional (status) override
        let (status, log_level) = if inner.peek().is_some_and(|t| {
            match t {
                TokenTree::Group(g) => g.delimiter() == proc_macro2::Delimiter::Parenthesis,
                _ => false,
            }
        }) {
            let tt = inner.next().unwrap();
            if let TokenTree::Group(g) = tt {
                parse_entry_parens(g)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Expect =>
        let tt = inner.next().expect("expected '=>'");
        match tt {
            TokenTree::Punct(ref p) => {
                if p.as_char() != '=' {
                    panic!("expected '=', got: {}", p.as_char());
                }
            }
            _ => panic!("expected '=>', got: {}", tt),
        }
        // Next must be '>'
        expect_punct(&mut inner, '>');

        // Parse detail string
        let detail = match inner.next() {
            Some(TokenTree::Literal(lit)) => {
                let s = lit.to_string();
                // Remove surrounding quotes
                s[1..s.len() - 1].to_string()
            }
            Some(tt) => panic!("expected string for detail, got: {}", tt),
            None => panic!("unexpected end of input, expected string for detail"),
        };

        // Eat optional trailing comma
        if let Some(TokenTree::Punct(p)) = inner.peek()
            && p.as_char() == ','
        {
            inner.next();
        }

        entries.push(Entry {
            name,
            status,
            log_level,
            detail,
            has_param,
        });
    }

    entries
}

fn parse_u16(tokens: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>) -> u16 {
    match tokens.next() {
        Some(TokenTree::Literal(lit)) => {
            let s = lit.to_string();
            s.parse().unwrap_or_else(|_| panic!("expected u16 number, got: {}", s))
        }
        Some(tt) => panic!("expected u16 number, got: {}", tt),
        None => panic!("unexpected end of input, expected u16"),
    }
}

fn parse_entry_parens(group: proc_macro2::Group) -> (Option<u16>, Option<String>) {
    let inner_tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut inner = inner_tokens.into_iter().peekable();

    let status = parse_u16(&mut inner);

    let log_level = if inner.peek().is_some() {
        expect_punct(&mut inner, ',');
        let kw = expect_ident(&mut inner, "log_level keyword").to_string();
        if kw.as_str() == "log_level" {
            expect_punct(&mut inner, ':');
            Some(expect_ident(&mut inner, "log_level value").to_string())
        } else {
            panic!("expected 'log_level', got: {}", kw);
        }
    } else {
        None
    };

    (Some(status), log_level)
}

fn collect_until(
    tokens: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>,
    delimiter: proc_macro2::Delimiter,
) -> TokenStream2 {
    let mut inner = TokenStream2::new();
    let mut depth = 1;
    for tt in tokens.by_ref() {
        match &tt {
            TokenTree::Group(g) if g.delimiter() == delimiter => {
                depth += 1;
                inner.extend(std::iter::once(tt));
            }
            TokenTree::Punct(p) => {
                let ch = p.as_char();
                if (delimiter == proc_macro2::Delimiter::Bracket && ch == '[')
                    || (delimiter == proc_macro2::Delimiter::Brace && ch == '{')
                    || (delimiter == proc_macro2::Delimiter::Parenthesis && ch == '(')
                {
                    depth += 1;
                } else if (delimiter == proc_macro2::Delimiter::Bracket && ch == ']')
                    || (delimiter == proc_macro2::Delimiter::Brace && ch == '}')
                    || (delimiter == proc_macro2::Delimiter::Parenthesis && ch == ')')
                {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                inner.extend(std::iter::once(tt));
            }
            _ => {
                inner.extend(std::iter::once(tt));
            }
        }
    }
    inner
}

#[proc_macro_derive(AutoFromPartial, attributes(auto_from))]
pub fn derive_auto_from(input: TokenStream) -> TokenStream {
    // Parse input as DeriveInput
    let input = parse_macro_input!(input as DeriveInput);

    // Extract destination struct name
    let dest_name = &input.ident;

    // Find #[auto_from(...)] attribute
    let source_type = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("auto_from"))
        .and_then(|attr| {
            let meta = attr.parse_args::<syn::Type>().ok()?;
            Some(meta)
        })
        .expect("Missing #[auto_from(SourceType)] attribute");

    // Extract destination struct fields
    let dest_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Generate tokens for field copying
    let field_assignments = dest_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: source.#field_name
        }
    });

    // Generate From implementation
    let expanded = quote! {
        impl From<#source_type> for #dest_name {
            fn from(source: #source_type) -> Self {
                Self {
                    #(#field_assignments,)*
                }
            }
        }
    };

    // Return generated code as TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(AutoToFull, attributes(to_full, default_values, new))]
pub fn derive_auto_to_full(input: TokenStream) -> TokenStream {
    // Parse input as DeriveInput
    let input = parse_macro_input!(input as DeriveInput);

    // Partial struct name
    let partial_name = &input.ident;

    // Trova l'attributo #[to_full(...)]
    let full_type = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("to_full"))
        .and_then(|attr| {
            let meta = attr.parse_args::<syn::Type>().ok()?;
            Some(meta)
        })
        .expect("Missing #[to_full(FullType)] attribute");

    // Extract partial struct fields
    let partial_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Find any default values specified
    let default_values: HashMap<String, TokenStream2> = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("new"))
        .map(|attr| {
            let content = attr.meta.require_list().unwrap().tokens.clone();
            parse_default_values(content)
        })
        .unwrap_or_default();

    // Generate partial fields to copy
    let partial_field_names = partial_fields
        .iter()
        //.filter(|field| !is_option_field(&field))
        .filter_map(|field| field.ident.as_ref().map(|ident| ident.to_string()));

    /*
    let partial_options_field_names = partial_fields
        .iter()
        .filter(|field| is_option_field(&field))
        .filter_map(|field| field.ident.as_ref().map(|ident| ident.to_string()));
    */

    // Generate tokens for implementation
    let field_assignments = {
        let copy_partial_fields = partial_field_names.clone().map(|field_name| {
            let field_ident = format_ident!("{}", field_name);
            quote! {
                full.#field_ident = self.#field_ident;
            }
        });

        /*
        let copy_partial_options_fields = partial_options_field_names.clone().map(|field_name| {
            let field_ident = format_ident!("{}", field_name);
            quote! {
                full.#field_ident = Some(self.#field_ident);
            }
        });
        */

        let set_default_values = default_values.iter().map(|(field_name, default_value)| {
            let field_ident = format_ident!("{}", field_name);
            quote! {
                full.#field_ident = #default_value;
            }
        });

        quote! {
            // Start with default values for the full struct
            let mut full = <#full_type as BaseModel>::new();

            // Override with partial struct fields
            #(#copy_partial_fields)*

            //#(#copy_partial_options_fields)*

            // Override with any specified default values
            #(#set_default_values)*

            full
        }
    };

    // Generate To implementation
    let expanded = quote! {
        impl To<#full_type> for #partial_name {
            fn to(self) -> #full_type {
                #field_assignments
            }
        }
    };

    // Return generated code as TokenStream
    TokenStream::from(expanded)
}

#[allow(dead_code)]
fn is_option_field(field: &Field) -> bool {
    match &field.ty {
        syn::Type::Path(type_path) => {
            let path = &type_path.path;
            if path.segments.len() == 1 {
                let segment = &path.segments[0];
                segment.ident == "Option"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn parse_default_values(tokens: TokenStream2) -> HashMap<String, TokenStream2> {
    let content = tokens.to_string();
    let mut defaults = HashMap::new();

    for pair in content.split(',') {
        if let Some((key, value)) = pair.split_once('=') {
            let key = key.trim().to_string();
            let value_tokens: TokenStream2 = value.trim().parse().unwrap();
            defaults.insert(key, value_tokens);
        }
    }

    defaults
}