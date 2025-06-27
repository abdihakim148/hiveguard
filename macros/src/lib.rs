use syn::{parse_macro_input, parse_str, FnArg, Ident, ItemTrait, Pat, TraitItem, Receiver, parse2, TraitItemFn};
use std::collections::HashMap;
use proc_macro::TokenStream;
use quote::quote;

use crate::io::{add_table, delete_tables_file};


mod io;

enum Field<R, I> {
    Receiver(R),
    Ident(I),
}


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
    let mut item = parse_macro_input!(input as ItemTrait);
    let items = item.items.as_slice();
    let client = match get_client_method_name(items) {
        Some(name) => name,
        None => {
            let err_msg = "there is no method with the client attribute. make sure to have a client method";
            return quote!{compile_error!(#err_msg)}.into();
        }
    };
    let traits = io::get_tables();
    let tables = match traits_to_methods(traits){
        Ok(map) => map,
        Err(stream) => return stream
    };
    let map = get_table_calling_methods(items, tables);
    let mut all_methods = Vec::new();
    for (_, (table, methods)) in map {
        for method in methods {
            let method_name = &method.sig.ident;
            let args = method.sig.inputs.into_iter().collect::<Vec<_>>();
            let outputs = method.sig.output;
            let (async_init, async_end) = if let Some(_) = method.sig.asyncness {(quote!{async}, quote!{.await})} else {(quote!{}, quote!{})};
            if let Some((args, fields)) = formated_args_and_fields(args.as_slice(), &client) {
                let mut mutability = false;
                let fields = fields.into_iter().filter_map(|field|{
                    match field {
                        Field::Receiver(receiver) => {
                            if let Some(_) = receiver.mutability {
                                mutability = true;
                            }
                            None
                        },
                        Field::Ident(ident) => Some(ident)
                    }
                }).collect::<Vec<_>>();
                let mutable = if mutability {quote!{mut}} else {quote!{}};
                let method = quote!{
                    #async_init fn #method_name(#(#args),*) #outputs {
                        let #mutable #client = self.#client();
                        let table = self.#table();
                        table.#method_name(#(#fields),*)#async_end
                    }
                };
                let trait_method: TraitItemFn = parse2(method).expect("THIS IS NOT SUPPOSED TO HAPPEN");
                let item = TraitItem::Fn(trait_method);
                all_methods.push(item);
            }
        }
    }
    for method in all_methods {
        item.items.push(method);
    }
    delete_tables_file().expect("could not delete the created table file");
    quote!{#item}.into()
}


fn get_client_method_name(items: &[TraitItem]) -> Option<Ident> {
    for item in items {
        if let TraitItem::Fn(method) = item {
            let name = method.sig.ident.clone();
            for attribute in &method.attrs {
                if let Some(ident) = attribute.path().get_ident() {
                    if ident.to_string() == "client" {
                        return Some(name)
                    }
                }
            }
        }
    }
    None
}


fn formated_args_and_fields<'a>(args: &'a [FnArg], client: &'a Ident) -> Option<(Vec<&'a FnArg>, Vec<Field<&'a Receiver, &'a Ident>>)> {
    let mut arguments = Vec::new();
    let mut fields = Vec::new();
    let mut receives = false;
    for arg in args {
        let field_name = argument_field_name(arg);
        if let Some(field) = field_name {
            match field {
                Field::Receiver(_) => {
                    receives = true;
                },
                Field::Ident(name) => {
                    if *name == *client {
                        fields.push(field);
                        continue;
                    }
                },
            }
            fields.push(field);
            arguments.push(arg);
        }
    }
    if !receives {
        return None;
    }
    Some((arguments, fields))
}

fn argument_field_name<'a>(arg: &'a FnArg) -> Option<Field<&'a Receiver, &'a Ident>> {
    match arg {
        FnArg::Receiver(receiver) => {
            Some(Field::Receiver(receiver))
        },
        FnArg::Typed(typed) => {
            if let Pat::Ident(pat_ident) = &*typed.pat {
                return Some(Field::Ident(&pat_ident.ident))
            }
            None
        }
    }
}


fn get_table_calling_methods(items: &[TraitItem], mut tables: HashMap<Ident, Vec<TraitItemFn>>) -> HashMap<Ident, (&Ident, Vec<TraitItemFn>)> {
    let mut map = HashMap::new();
    for item in items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();
            let table_name = snake_to_pascal_case(&method_name_str);
            let table_ident = Ident::new(&table_name, method_name.span());
            if let Some(methods) = tables.remove(&table_ident) {
                map.insert(table_ident, (method_name, methods));
            }
        }
    }
    map
}


fn traits_to_methods(traits: HashMap<String, String>) -> Result<HashMap<Ident, Vec<TraitItemFn>>, TokenStream> {
    let mut all_methods = HashMap::new();
    let mut map = HashMap::new();
    for (_, input) in traits {
        let item = parse_str::<ItemTrait>(&input).expect("could not parse trait item");
        let trait_name = item.ident;
        let mut methods = Vec::new();
        for item in item.items {
            if let TraitItem::Fn(method) = item {
                let method_name = method.sig.ident.clone();
                if let Some(name) = map.get(&method_name) {
                    return Err(quote!{compile_error!(concat!("Method ", stringify!(#method_name), " is defined in both ", stringify!(#name), " and ", stringify!(#trait_name), " traits. try changing one method name"))}.into());
                }
                map.insert(method_name, trait_name.clone());
                methods.push(method);
            }
        }
        all_methods.insert(trait_name, methods);
    }
    Ok(all_methods)
}



fn snake_to_pascal_case(input: &str) -> String {
    input
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}