use syn::{FnArg, ReturnType, Ident, Generics, TraitItem, TraitItemFn, Type, TypePath, parse2, Pat, PathArguments, Attribute, Meta};
use proc_macro2::TokenStream;
use super::{transform_type_path_in_place, Result};
use quote::quote;
use std::collections::HashSet;


pub struct Method {
    /// The name of the method, e.g., `create_session`
    name: Ident,
    /// Whether the method returns a Future
    /// This is used to determine if the method should be awaited.
    future: bool,
    /// The arguments of the method, e.g., `session: Session, client: &Client`
    args: Vec<FnArg>,
    /// The generics of the method, e.g., `<Client>`
    generics: Generics,
    /// The return type of the method, e.g., `Result<(), Self::Error>`
    outputs: ReturnType,
    fields: Vec<Pat>,
    /// Identifiers to skip during transformation
    skip: HashSet<String>
}

impl Method {
    fn remove_arg(&mut self, arg_name: &Ident) {
        // Remove the client argument from the method arguments
        self.args.retain(|arg| {
            if let Some(name) = Self::field_name(arg) {
                // If the argument is the specified one, remove it
                name != *arg_name
            } else {
                // If it's not a named argument, keep it
                true
            }
        });
    }

    fn update_type(ty: &mut Type, type_ident: Ident, (trait_ident, trait_args): (&Ident, &PathArguments), skip: &HashSet<String>) {
        if let Type::Path(type_path) = ty {
            transform_type_path_in_place(type_path, type_ident, (trait_ident, trait_args), skip);
        }
    }

    fn update_return(&mut self, type_name: Ident, (trait_ident, trait_args): (&Ident, &PathArguments)) {
        if let ReturnType::Type(_, ty) = &mut self.outputs {
            Method::update_type(ty, type_name, (trait_ident, trait_args), &self.skip);
        }
    }

    fn update_args(&mut self, type_ident: Ident, (trait_ident, trait_args): (&Ident, &PathArguments)) {
        // Update the arguments to include the table type
        for arg in &mut self.args {
            if let FnArg::Typed(pat_type) = arg {
                if let Some(type_path) = get_path_from_type(&mut pat_type.ty) {
                    transform_type_path_in_place(type_path, type_ident.clone(), (trait_ident, trait_args), &self.skip);
                }
            }
        }
    }

    fn future(&self) -> (TokenStream, TokenStream) {
        if self.future {
            (quote! {async}, quote! {.await})
        }else {
            (quote! {}, quote! {})
        }
    }

    fn field_name(argument: &FnArg) -> Option<Ident> {
        if let FnArg::Typed(pat_type) = argument {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                return Some(pat_ident.ident.clone());
            }
        }
        None
    }

    fn fields_from_args(args: &[FnArg]) -> Vec<Pat> {
        args.iter().filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type.pat.as_ref())
            } else {
                None
            }
        }).cloned().collect()
    }

    /// Parse skip attribute to extract identifiers to skip during transformation
    fn parse_skip_attribute(attrs: &[Attribute]) -> HashSet<String> {
        let mut skip_set = HashSet::new();
        
        for attr in attrs {
            if attr.path().is_ident("skip") {
                if let Meta::List(meta_list) = &attr.meta {
                    for token in meta_list.tokens.clone() {
                        // Parse each token as an identifier
                        if let Ok(ident) = syn::parse2::<Ident>(token.into()) {
                            skip_set.insert(ident.to_string());
                        }
                    }
                }
            }
        }
        
        skip_set
    }

    pub fn to_tokens(&mut self, table_method_name: &Ident, client_name: &Ident, table_type_name: &Ident, (trait_name, trait_args): (&Ident, &PathArguments)) -> Result<TraitItem> {
        // Remove the client argument from the method arguments
        self.remove_arg(&client_name);

        // Update the return type to include the table type
        self.update_return(table_type_name.clone(), (trait_name, trait_args));

        // Update the arguments to include the table type
        self.update_args(table_type_name.clone(), (trait_name, trait_args));

        let (async_kw, await_kw) = self.future();
        let name = &self.name;
        let args = &self.args;
        let generics = &self.generics;
        let outputs = &self.outputs;
        let fields = &self.fields;

        // Generate the method signature
        let method = quote! {
            #async_kw fn #name #generics(#(#args),*) #outputs {
                let #client_name = self.#client_name();
                let table = self.#table_method_name();
                table.#name(#(#fields),*)#await_kw.map_err(Into::into)
            }
        };
        let method = parse2(method)?;
        Ok(TraitItem::Fn(method))
    }
}


impl From<TraitItemFn> for Method {
    fn from(method_item: TraitItemFn) -> Self {
        let name = method_item.sig.ident;
        let future = method_item.sig.asyncness.is_some();
        let args = method_item.sig.inputs.into_iter().collect::<Vec<FnArg>>();
        let generics = method_item.sig.generics.clone();
        let outputs = method_item.sig.output.clone();
        let fields = Method::fields_from_args(&args);
        let skip = Method::parse_skip_attribute(&method_item.attrs);

        Self { name, future, args, generics, outputs, fields, skip }
    }
}


fn get_path_from_type(ty: &mut Type) -> Option<&mut TypePath> {
    match ty {
        Type::Path(type_path) => {
           Some(type_path)
        },
        Type::Reference(reference) => get_path_from_type(&mut reference.elem),
        _ => None,
    }
}