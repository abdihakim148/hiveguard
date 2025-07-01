use syn::{Ident, ItemTrait, PathArguments, TraitItem};
use super::{Method, Result};

pub struct Table {
    /// The name of the table trait, e.g., `UsersTable`
    name: Ident,
    /// The methods defined in the table trait
    methods: Vec<Method>
}


impl Table {
    fn new(trait_item: ItemTrait) -> Self {
        let name = trait_item.ident;
        let methods = Self::extract_methods(trait_item.items);
        Self{name, methods}
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    fn extract_methods(trait_item: Vec<TraitItem>) -> Vec<Method> {
        let mut methods = Vec::new();
        for item in trait_item {
            if let TraitItem::Fn(method) = item {
                methods.push(method.into());
            }
        }
        methods
    }

    pub fn methods(&mut self, client: &Ident, table_method_ident: &Ident, table_type_ident: &Ident, table_trait_ident: &Ident, table_trait_args: &PathArguments) -> Result<Vec<TraitItem>> {
        let mut items = Vec::new();
        for method in &mut self.methods {
            let item = method.to_tokens(table_method_ident, client, table_type_ident, (table_trait_ident, table_trait_args))?;
            items.push(item);
        }
        Ok(items)
    }
}


impl From<ItemTrait> for Table {
    fn from(trait_item: ItemTrait) -> Self {
        Self::new(trait_item)
    }
}