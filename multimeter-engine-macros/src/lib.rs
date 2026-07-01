use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, parse_macro_input};

/// to generate something like
/// ```rust,ignore
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
/// ```rust,ignore
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
        Data::Struct(s) => match s.fields {
            Fields::Named(n) => n.named,
            Fields::Unit => Default::default(),
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "This macro only supports structs with named fields or unit structs",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "This macro only supports struct with fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut item_statements = Vec::new();
    let mut nest_statements = Vec::new();

    for field in fields {
        let field_name = match field.ident {
            Some(n) => n,
            None => continue,
        };

        for attr in field.attrs.iter().filter(|e| e.path().is_ident("query")) {
            let mut state = 0u8; // 1 for key, 2 for nest
            let mut query_variable_name: Option<String> = None;
            let mut calling_function_name: Option<Ident> = None;
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("key") {
                    if state == 1 {
                        return Err(meta.error("Duplicate key is not allowed"));
                    }
                    if state == 2 {
                        return Err(meta.error("Key and nest should not appear both"));
                    }
                    state = 1;
                    query_variable_name = Some(meta.value()?.parse::<LitStr>()?.value());

                    Ok(())
                } else if meta.path.is_ident("nest") {
                    if state == 2 {
                        return Err(meta.error("Duplicate nest is not allowed"));
                    }
                    if state == 1 {
                        return Err(meta.error("Key and nest should not appear both"));
                    }
                    state = 2;

                    Ok(())
                } else if meta.path.is_ident("function") {
                    let function_name = meta.value()?.parse::<LitStr>()?;
                    calling_function_name = Some(function_name.parse()?);

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

            if state == 2 {
                nest_statements.push(quote! {
                    match crate::monitor::query::QueryField::query(&self.#field_name, key, None) {
                        crate::monitor::query::QueryResult::NotFound => {},
                        f => return f,
                    }
                });
            } else {
                if let Some(query_variable_name) = query_variable_name {
                    if let Some(calling_function_name) = calling_function_name {
                        item_statements.push(quote! {
                        #query_variable_name => return Self::#calling_function_name(&self, attach),
                    });
                    } else {
                        item_statements.push(quote! {
                        #query_variable_name =>  return crate::monitor::query::QueryResult::Found(
                            self.#field_name.clone().map(crate::util::data_container::DataContainer::from),
                        ),
                    })
                    }
                }
            }
        }
    }

    let full_statement = quote! {
        impl crate::monitor::query::QueryField for #struct_name {
            fn query(&self, key: &str, attach: Option<&crate::util::info_map::InfoMap>) -> crate::monitor::query::QueryResult {
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
                #[query(key = "cpu_name", function = "get_cpu_name")]
                #[query(key = "cpu_name", function = "get_cpu_name")]
                pub name: Option<String>,             // name of the CPU
                pub usage: Option<f64>,               // cpu usage
                pub package_temperature: Option<f64>, // package temperature
            }
        };

        println!("{:#?}", input);
    }
}
