use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
