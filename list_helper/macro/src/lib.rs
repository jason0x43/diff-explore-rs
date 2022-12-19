use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ListCursor)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl ListCursor for #ident {
            fn cursor(&self) -> usize {
                self.list.cursor()
            }

            fn mut_list(&mut self) -> &mut ListData {
                self.list.mut_list()
            }
        }
    };
    output.into()
}
