use syn::Ident;

pub mod models;
pub mod schema;

pub fn append_ident(ident: &Ident, string: &str) -> Ident {
    let mut out = ident.to_string();
    out.push_str(string);
    Ident::new(&out, proc_macro2::Span::call_site())
}
