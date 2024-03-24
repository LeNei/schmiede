use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, Type};

#[proc_macro_derive(Displayable, attributes(serde))]
pub fn derive_displayable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let mut columns = vec![];
    let mut values = vec![];
    let mut filter = quote!();
    let syn::DeriveInput { data, ident, .. } = ast;

    let schema_name = ident.to_string().to_lowercase();
    let schema_ident = Ident::new(&schema_name, Span::call_site());

    if let syn::Data::Struct(syn::DataStruct { fields, .. }) = data {
        'field_loop: for field in fields {
            for attr in &field.attrs {
                if attr.path().is_ident("serde") {
                    let meta = attr.parse_args::<Meta>().unwrap();
                    match meta {
                        Meta::Path(path) => {
                            let ident = path.get_ident().unwrap().to_string();
                            if ident == "skip_serializing" {
                                continue 'field_loop;
                            }
                        }
                        Meta::List(_) => {
                            println!("cargo:warning=Found serde list for field");
                        }
                        Meta::NameValue(_) => {
                            println!("cargo:warning=Found serde name value for field");
                        }
                    }
                }
            }
            let field_name = &field.ident.clone().unwrap().to_string();
            let value_ident = &field.ident;
            columns.push(quote! (#field_name));
            values.push(quote!(self.#value_ident.to_string()));
            if is_string_type(&field.ty) {
                if filter.is_empty() {
                    filter = quote!(#schema_ident::#value_ident.ilike(filter.clone()));
                } else {
                    filter = quote!(#filter.or(#schema_ident::#value_ident.ilike(filter.clone())));
                }
            }
        }
    }

    quote!(
        use crate::schema::#schema_ident;
        impl Displayable for #ident {
            type SqlType = diesel::dsl::SqlTypeOf<diesel::dsl::AsSelect<Self, diesel::pg::Pg>>;
            type BoxedQuery<'a> = #schema_ident::BoxedQuery<'a, diesel::pg::Pg, Self::SqlType>;

            fn table_headers() -> Vec<&'static str> {
                vec![#(#columns),*]
            }

            fn to_table_row(&self) -> Vec<String> {
                vec![#(#values),*]
            }

            fn all() -> Self::BoxedQuery<'static> {
                #schema_ident::table.select(Self::as_select()).into_boxed()
            }

            fn paginated<'a>(offset: i64, page_size: i64, search_term: Option<String>) -> Self::BoxedQuery<'a> {
                if let Some(filter) = search_term {
                    let filter = format!("%{}%", filter.to_lowercase());
                    Self::all()
                        .filter(
                            #filter
                        )
                        .offset(offset)
                        .limit(page_size)
                } else {
                    Self::all()
                        .select(Self::as_select())
                        .offset(offset)
                        .limit(page_size)
                }
            }
        }
    )
    .into()
}

fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        // Assumes a simple 'String' type - adjust if necessary
        type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "String"
    } else {
        false
    }
}
