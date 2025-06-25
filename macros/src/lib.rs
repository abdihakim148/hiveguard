use syn::{parse_macro_input, parse_str, FnArg, Ident, ItemTrait, Pat, ReturnType, TraitItem, Type, TypeParamBound, parse2};
use std::collections::HashMap;
use proc_macro::TokenStream;
use quote::quote;

use crate::io::delete_tables_file;


mod io;


#[proc_macro_attribute]
pub fn client(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn table(_: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemTrait);
    let trait_name = item.ident.to_string();
    let items = item.items.as_slice();
    let methods = get_methods_from_trait_item(items);
    if !methods.is_empty() {
        io::add_table(trait_name, methods)
    }
    quote!{#item}.into()
}


#[proc_macro_attribute]
pub fn database(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemTrait);
    let items = item.items.as_slice();
    let client_method_name = match get_client_method_name(items) {
        Some(name) => name,
        None => {
            let err_msg = "there is no method with the client attribute. make sure to have a client method";
            return quote!{compile_error!(#err_msg)}.into();
        }
    };
    let tables = io::get_tables();
    let map = get_table_calling_methods(items, tables);
    let mut full_methods = Vec::new();
    let set = HashMap::<String, String>::new();
    for (trait_name, (table, methods)) in map {
        for method in methods {
            let name = method.name;
            let ident: Ident = parse_str(&name).expect("could not parse method name into an ident");
            if let Some(existing_trait_name) = set.get(&name) {
                let err_msg = format!("Method {} is defined in both {} and {} traits. try changing one name", name, existing_trait_name, trait_name);
                return quote!{compile_error!(#err_msg)}.into();
            }
            let args = method.args.iter().map(|arg|{
                parse_str(arg).expect("could not parse arg into a type")
            }).collect::<Vec<FnArg>>();
            let output: ReturnType = parse_str(&method.outputs).expect("could not parse output into the ReturnType");
            let ((before_args, before_fields), (after_args, after_fields)) = arguments_without_client(args, &client_method_name);
            let (async_init, async_end) = if method.future {(quote!{async}, quote!{.await})}else{(quote!{}, quote!())};
            let method = quote!{
                #async_init fn #ident(&self, #(#before_args),*, #(#after_args),*) #output {
                    let table = self.#table();
                    let client = &self.#client_method_name();
                    table.#ident(#(#before_fields), *, client, #(#after_fields), *)#async_end
                }
            };
            // let method: TraitItem = parse2(method).expect("THIS IS NOT SUPPOSED TO HAPPEN");
            full_methods.push(method);
        }
    }
    for method in full_methods {
        // item.items.push(method);
        println!("{}", method);
    }
    delete_tables_file().expect("could not delete the created table file");
    quote!{#item}.into()
}



fn get_methods_from_trait_item(items: &[TraitItem]) -> Vec<io::Method> {
    let mut methods = Vec::new();
    for item in items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let arguments = &method.sig.inputs;
            let mut receiver = false;
            let future = if let Some(_) = &method.sig.asyncness{true}else{false};
            let arguments = arguments.iter().filter_map(|argument|{
                match argument {
                    FnArg::Receiver(_) => {receiver = true; None},
                    FnArg::Typed(typed) => Some(quote!{#typed}.to_string()),
                }
            }).collect::<Vec<_>>();
            if !receiver {
                continue;
            }
            let output = &method.sig.output;
            let name = quote!{#method_name}.to_string();
            let args = arguments;
            let outputs = quote!{#output}.to_string();
            let method = io::Method{
                future,
                name,
                args,
                outputs
            };
            methods.push(method);
        }
    }
    methods
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


fn arguments_without_client(args: Vec<FnArg>, client: &Ident) -> ((Vec<FnArg>, Vec<Ident>), (Vec<FnArg>, Vec<Ident>)) {
    let (mut before_args, mut before_fields) = (Vec::new(), Vec::new());
    let (mut after_args, mut after_fields) = (Vec::new(), Vec::new());
    let mut client_found = false;
    for arg in args {
        let name = match argument_field_name(&arg) {
            Some(name) => name,
            None => continue,
        };
        if *name == *client {
            client_found = true;
            continue;
        }
        if client_found {
            after_fields.push(name.clone());
            after_args.push(arg);
        } else {
            before_fields.push(name.clone());
            before_args.push(arg);
        }
    }
    ((before_args, before_fields), (after_args, after_fields))
}


fn argument_field_name(arg: &FnArg) -> Option<&Ident> {
    match arg {
        FnArg::Receiver(_) => None,
        FnArg::Typed(typed) => {
            if let Pat::Ident(pat_ident) = &*typed.pat {
                return Some(&pat_ident.ident)
            }
            None
        }
    }
}


fn get_table_calling_methods(items: &[TraitItem], mut tables: HashMap<String, Vec<io::Method>>) -> HashMap<String, (&Ident, Vec<io::Method>)> {
    let mut map = HashMap::new();
    for item in items {
        if let TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let output = &method.sig.output;
            if let ReturnType::Type(_, typ) = output {
                let mut typ = typ.as_ref();
                if let Type::Reference(ty) = typ {
                    typ = ty.elem.as_ref();
                }
                if let Type::ImplTrait(type_impl_trait) = typ {
                    for bound in &type_impl_trait.bounds {
                        if let TypeParamBound::Trait(trait_bound) = bound {
                            let path = &trait_bound.path;
                            if let Some(segment) = path.segments.last() {
                                let ident = segment.ident.clone();
                                let name = ident.to_string();
                                if let Some(methods) = tables.remove(&name) {
                                    map.insert(name, (method_name, methods));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}