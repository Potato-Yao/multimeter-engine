use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input, Fields, LitStr};

/// to generate something lik
/// ```rust
/// impl crate::monitor::query::QueryField for CPU {
///     fn query(&self, key: &str) -> crate::monitor::query::QueryResult {
///         match key {
///             "name" => crate::monitor::query::QueryResult::Found(
///                 self.name.clone().map(|e| crate::monitor::DataContainer::from(e)),
///             ),
///             _ => crate::monitor::query::QueryResult::NotFound,
///         }
///     }
/// }
/// ```
/// and
/// ```rust
/// impl QueryField for Device {
///     fn query(&self, key: &str) -> QueryResult {
///         match key {
///             _ => {}
///         }
///
///         match self.cpu.query(key) {
///             QueryResult::NotFound => {},
///             f => return f,
///         }
///
///         QueryResult::NotFound
///     }
/// }
/// ```
#[proc_macro_derive(QueryGenerator, attributes(query))]
pub fn query_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => {
            match s.fields {
                Fields::Named(n) => n.named,
                _ => {
                    return syn::Error::new_spanned(struct_name, "This macro only supports struct with fields").to_compile_error().into();
                }
            }
        },
        _ => {
            return syn::Error::new_spanned(struct_name, "This macro only supports struct with fields").to_compile_error().into();
        }
    };

    let mut item_statements = Vec::new();
    let mut nest_statements = Vec::new();

    for field in fields {
        let field_name = match field.ident {
            Some(n) => n,
            None => continue,
        };

        // this shit cannot be collected. so use iter
        for attr in field.attrs.iter().filter(|e| e.path().is_ident("query")) {
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("key") {
                    let attr_value = meta.value()?.parse::<LitStr>()?.value();
                    item_statements.push(quote! {
                        #attr_value =>  return crate::monitor::query::QueryResult::Found(
                            self.#field_name.clone().map(|e| crate::monitor::DataContainer::from(e)),
                        ),
                    });

                    Ok(())
                } else if meta.path.is_ident("nest") {
                    nest_statements.push(quote! {
                        match self.#field_name.crate::monitor::query(key) {
                            crate::monitor::query::QueryResult::NotFound => {},
                            f => return f,
                        }
                    });

                    Ok(())
                } else if meta.path.is_ident("skip") {
                    Ok(())
                } else {
                    Err(meta.error("Unsupported attribute value of query"))
                }
            });

            if let Err(e) = result {
                return e.to_compile_error().into();
            }
        }
    }

    let full_statement = quote! {
        impl crate::monitor::query::QueryField for #struct_name {
            fn query(&self, key: &str) -> crate::monitor::query::QueryResult {
                match key {
                    #(#item_statements)*
                    _ => {}
                }

                #(#nest_statements)*

                crate::monitor::query::QueryResult::NotFound
            }
        }
    };

    full_statement.into()
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_syn() {
        let input: DeriveInput = parse_quote! {
            #[allow(clippy::upper_case_acronyms)]
            #[derive(Default, Debug, Clone)]
            pub struct CPU {
                #[query(key = "cpu_name")]
                pub name: Option<String>,             // name of the CPU
                pub usage: Option<f64>,               // cpu usage
                pub package_temperature: Option<f64>, // package temperature
            }
        };

        println!("{:#?}", input);
    }
}
