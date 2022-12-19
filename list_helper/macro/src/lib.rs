use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, DeriveInput};

/// Add list data to a struct
#[proc_macro_attribute]
pub fn list_data(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            // Add a new named field with name 'list'
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! {
                                list: ListData
                            })
                            .unwrap(),
                    );
                }
                _ => (),
            }

            return quote! {
                #ast
            }
            .into();
        }
        _ => panic!("`add_field` has to be used with structs "),
    }
}

#[proc_macro_attribute]
pub fn list_count(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            // Add a new named field with name 'list'
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! {
                                list_state: ListState
                            })
                            .unwrap(),
                    );
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! {
                                list_height: usize
                            })
                            .unwrap(),
                    );
                }
                _ => (),
            }

            return quote! {
                #ast
            }
            .into();
        }
        _ => panic!("`add_field` has to be used with structs "),
    }
}

#[proc_macro_derive(ListCursor)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl ListCursor for #ident {
            fn cursor(&self) -> usize {
                self.list.state.selected().unwrap_or(0)
            }

            fn list_state(&mut self) -> &mut ListState {
                &mut self.list.state
            }

            fn list_count(&self) -> usize {
                self.list.count
            }

            fn set_list_count(&mut self, count: usize) {
                self.list.count = count
            }

            fn list_height(&self) -> usize {
                self.list.height
            }

            fn set_list_height(&mut self, height: usize) {
                self.list.height = height
            }
        }
    };
    output.into()
}
