use syn::{parse_macro_input, ItemTrait};
use proc_macro::TokenStream;
use quote::quote;

use crate::io::{add_table, delete_tables_file};


mod io;
mod table;
mod database;


#[proc_macro_attribute]
pub fn client(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}


#[proc_macro_attribute]
pub fn table(_: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemTrait);
    let trait_name = item.ident.to_string();
    let value = quote!{#item}.to_string();
    add_table(trait_name, value);
    quote!{#item}.into()
}


#[proc_macro_attribute]
pub fn database(_: TokenStream, input: TokenStream) -> TokenStream {
    let database: database::Database = match input.try_into() {
        Ok(db) => db,
        Err(err) => {
            let err_msg = format!("Error parsing Database trait: {}", err);
            return quote! {
                compile_error!(#err_msg);
            }
            .into();
        }
    };
    let tables = io::get_tables();
    delete_tables_file().expect("Could not delete the tables file");
    let expanded = database.expand(tables);
    match expanded {
        Ok(tokens) => tokens.into(),
        Err(err) => {
            let err_msg = format!("Error expanding database macro: {}", err);
            quote! {
                compile_error!(#err_msg);
            }
            .into()
        }
    }
}
