use crate::visitors::{
    key_finder::{get_schema_field_keys, get_schema_outer_key},
    utils::KeyInfo,
};
use syn::ExprClosure;

use crate::visitors::{schema_finder::SchemaType, utils::KeyType};

pub struct KeyValidator<'schema> {
    pub info: KeyInfo<'schema>,
    pub valid_key: bool,
}

pub(crate) fn check_outer_key<'ast: 'b, 'b>(
    closure: &'b ExprClosure,
    schema: &'b SchemaType<'ast>,
) -> bool {
    fn check_expr_input_type<'a>(expr: &ExprClosure, schema: &SchemaType<'a>) -> bool {
        expr.inputs.iter().any(|inp| {
            if let syn::Pat::Type(pat_type) = inp
                && let syn::Type::Path(type_path) = pat_type.ty.as_ref()
            {
                type_path.path.is_ident(schema.identity())
            } else {
                panic!(
                    "Closure input value must be of type {:?}",
                    schema.identity().to_string()
                );
                false
            }
        })
    }

    if check_expr_input_type(closure, schema) {
        true
    } else {
        panic!("Closure must accept ");
        false
    }
}
