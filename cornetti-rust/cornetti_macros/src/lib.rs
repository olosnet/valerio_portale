use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

#[proc_macro_derive(AutoFromPartial, attributes(auto_from))]
pub fn derive_auto_from(input: TokenStream) -> TokenStream {
    // Parse l'input come una DeriveInput
    let input = parse_macro_input!(input as DeriveInput);

    // Estrai il nome della struct di destinazione
    let dest_name = &input.ident;

    // Trova l'attributo #[auto_from(...)]
    let source_type = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("auto_from"))
        .and_then(|attr| {
            let meta = attr.parse_args::<syn::Type>().ok()?;
            Some(meta)
        })
        .expect("Attributo #[auto_from(SourceType)] mancante");

    // Estrai i campi della struct di destinazione
    let dest_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Solo le struct con campi nominati sono supportate"),
        },
        _ => panic!("Solo le struct sono supportate"),
    };

    // Genera i token per copiare i campi
    let field_assignments = dest_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: source.#field_name
        }
    });

    // Genera l'implementazione di From
    let expanded = quote! {
        impl From<#source_type> for #dest_name {
            fn from(source: #source_type) -> Self {
                Self {
                    #(#field_assignments,)*
                }
            }
        }
    };

    // Restituisci il codice generato come TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(AutoToFull, attributes(to_full, default_values, new))]
pub fn derive_auto_to_full(input: TokenStream) -> TokenStream {
    // Parse dell'input come DeriveInput
    let input = parse_macro_input!(input as DeriveInput);

    // Nome della struct parziale
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
        .expect("Attributo #[to_full(FullType)] mancante");

    // Estrai i campi della struct parziale
    let partial_fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Solo le struct con campi nominati sono supportate"),
        },
        _ => panic!("Solo le struct sono supportate"),
    };

    // Trova eventuali valori di default specificati
    let default_values: HashMap<String, TokenStream2> = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("new"))
        .map(|attr| {
            let content = attr.meta.require_list().unwrap().tokens.clone();
            parse_default_values(content)
        })
        .unwrap_or_default();

    // Genera i campi parziali da copiare
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

    // Genera i token per l'implementazione
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
            // Inizia con i valori di default per la struct completa
            let mut full = <#full_type as BaseModel>::new();

            // Sovrascrivi con i campi della struct parziale
            #(#copy_partial_fields)*

            //#(#copy_partial_options_fields)*

            // Sovrascrivi con eventuali valori di default specificati
            #(#set_default_values)*

            full
        }
    };

    // Genera l'implementazione di To
    let expanded = quote! {
        impl To<#full_type> for #partial_name {
            fn to(self) -> #full_type {
                #field_assignments
            }
        }
    };

    // Restituisci il codice generato come TokenStream
    TokenStream::from(expanded)
}

#[allow(dead_code)]
fn is_option_field(field: &Field) -> bool {
    return match &field.ty {
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
    };
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