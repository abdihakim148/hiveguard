use syn::{parse, parse_str, Error, Ident, ItemTrait, PathArguments, ReturnType, TraitItem, TraitItemFn, TraitItemType, Type, TypeParamBound, TypePath};
use std::collections::HashMap;
use proc_macro::TokenStream;
use std::convert::TryFrom;
use super::table::Table;
use proc_macro2::Span;

pub struct Database {
    item: ItemTrait,
}

impl TryFrom<TokenStream> for Database {
    type Error = Error;

    fn try_from(tokens: TokenStream) -> Result<Self, Self::Error> {
        let item: ItemTrait = parse(tokens)?;
        Ok(Self { item })
    }
}

impl Database {
    pub fn expand(mut self, tables: HashMap<String, String>) -> Result<TokenStream, Error> {
        let client = self.client();
        let client = client.ok_or(Error::new(Span::call_site(), "No client found: make sure to give the client method the client attribute"))?;
        let mut trait_method_map = self.table_trait_and_method(&tables);
        let mut all_methods = Vec::new();
        for (_, table) in tables {
            let table_trait: ItemTrait = parse_str(&table)?;
            let mut table: Table = table_trait.into();
            let table_trait_ident = table.name().clone();
            if let Some((table_method_ident, table_type_ident, table_trait_args)) = trait_method_map.remove(&table_trait_ident) {
                let methods = table.methods(client, table_method_ident, table_type_ident, &table_trait_ident, table_trait_args)?; // Pass the table_trait_path
                all_methods.extend(methods);
            }
        }
        self.item.items.extend(all_methods);
        let item = self.item;
        Ok(quote::quote! {#item}.into())
    }

    
    fn methods(&self) -> Vec<&TraitItemFn> {
        let mut methods = Vec::new();
        for item in &self.item.items {
            if let syn::TraitItem::Fn(method) = item {
                methods.push(method);
            }
        }
        methods
    }

    fn types(&self) -> Vec<&TraitItemType> {
        let mut types = Vec::new();
        for item in &self.item.items {
            if let syn::TraitItem::Type(ty) = item {
                types.push(ty);
            }
        }
        types
    }

    fn client(&self) -> Option<&Ident> {
        for item in &self.item.items {
            if let TraitItem::Fn(method) = item {
                for attr in &method.attrs {
                    if let Some(ident) = attr.path().get_ident() {
                        return Some(ident);
                    }
                }
            }
        }
        None
    }

    fn table_trait_and_method(&self, tables: &HashMap<String, String>) -> HashMap<&Ident, (&Ident, &Ident, &PathArguments)> {
        let types = self.types();
        let mut types = types.into_iter().filter_map(|ty| {
            let type_name = &ty.ident;
            for bound in &ty.bounds {
                if let TypeParamBound::Trait(bound) = bound {
                    if let Some(segment) = bound.path.segments.last() {
                        let trait_name = &segment.ident;
                        let args = &segment.arguments;
                        if let Some(_) = tables.get(&trait_name.to_string()) {
                            // If the trait is in the tables map, we consider it valid
                            return Some((type_name, (trait_name, args)));
                        }
                    }
                }
            }
            None
        }).collect::<HashMap<&Ident, (&Ident, &PathArguments)>>();

        let mut map = HashMap::new();
        for method in self.methods() {
            let method_name = &method.sig.ident;
            if let ReturnType::Type(_, ty) = &method.sig.output {
                if let Some(path) = get_path_from_type(ty) {
                    if let Some(segment) = path.path.segments.last() {
                        let type_name = &segment.ident;
                        if let Some((trait_name, trait_args)) = types.remove(type_name) {
                            map.insert(trait_name, (method_name, type_name, trait_args));
                        }
                    }
                }
            }
        }

        map
    }
}


pub fn get_path_from_type(ty: &Type) -> Option<&TypePath> {
    match ty {
        Type::Path(type_path) => {
           Some(type_path)
        },
        Type::Reference(reference) => get_path_from_type(&reference.elem),
        _ => None,
    }
}