use proc_macro2::Span;
use syn::{
    DataStruct, Field, Ident, Item, ItemStruct, PathSegment, Token,
    punctuated::{Pair, Punctuated},
    visit::Visit,
};

#[derive(Default)]
pub struct SchemaVisitor<'ast> {
    pub items: Vec<(
        &'ast ItemStruct,
        Punctuated<PathSegment, Token![::]>,
        (&'ast Field, Option<&'ast Field>),
    )>,
    pub reg_name: Option<&'ast Ident>,
}

impl<'ast> Visit<'ast> for SchemaVisitor<'ast> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if let Some((_, content)) = &i.content {
            for item in content {
                let mut current_path: Vec<Pair<PathSegment, Token![::]>> = vec![];
                if let Item::Struct(item_struct) = item {
                    let mut final_path = current_path;
                    final_path.push(Pair::new(
                        PathSegment {
                            ident: item_struct.ident.clone(),
                            arguments: syn::PathArguments::None,
                        },
                        None,
                    ));
                    let final_path = Punctuated::from_iter(final_path);
                    let output = {
                        let mut primary = None;
                        let mut secondary = None;
                        item_struct.fields.iter().for_each(|field| {
                            if field
                                .attrs
                                .iter()
                                .any(|att| att.path().is_ident("primary_key"))
                            {
                                primary = Some(field);
                            } else if field
                                .attrs
                                .iter()
                                .any(|att| att.path().is_ident("secondary_key"))
                            {
                                secondary = None;
                            }
                        });
                        let primary = match primary {
                            Some(f) => f,
                            None => panic!("Primary key is necessary"),
                        };
                        (primary, secondary)
                    };
                    self.items.push((item_struct, final_path, output));
                } else if let Item::Mod(item_mod) = item {
                    current_path.push(Pair::new(
                        PathSegment {
                            ident: item_mod.ident.clone(),
                            arguments: syn::PathArguments::None,
                        },
                        Some(Token![::](Span::call_site())),
                    ));
                }
            }
        }
    }
}
