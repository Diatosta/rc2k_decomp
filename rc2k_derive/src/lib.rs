//! `#[derive(Cells)]` for `rc2k_hook::pointers::Cells`: a `#[repr(C)]` or `#[repr(transparent)]` record is made of cells
//! when each field is, and no byte lies outside its fields. Both are checked at compile time,
//! so the promise the trait asks for rests on the primitive cells alone. The derived `cell_at`
//! finds a byte's field by its offset and asks the field's type, down to a primitive cell.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Cells)]
pub fn derive_cells(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (with, of, clause) = input.generics.split_for_impl();
    let repr_c = input.attrs.iter().any(|a| {
        let text = quote!(#a).to_string();
        a.path().is_ident("repr") && (text.contains('C') || text.contains("transparent"))
    });
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.named.iter().collect::<Vec<_>>(),
            Fields::Unnamed(f) => f.unnamed.iter().collect(),
            Fields::Unit => Vec::new(),
        },
        _ => {
            return syn::Error::new_spanned(name, "Cells: only a struct")
                .to_compile_error()
                .into();
        }
    };
    if !repr_c {
        return syn::Error::new_spanned(name, "Cells: needs #[repr(C)] or #[repr(transparent)]")
            .to_compile_error()
            .into();
    }
    let types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let names: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| match &f.ident {
            Some(ident) => quote!(#ident),
            None => {
                let index = syn::Index::from(i);
                quote!(#index)
            }
        })
        .collect();
    let indices: Vec<_> = (0..fields.len()).collect();
    quote! {
        const _: () = {
            const fn is_cells<T: crate::pointers::Cells + ?Sized>() {}
            #( is_cells::<#types>(); )*
            assert!(
                core::mem::size_of::<#name #of>() == 0 #( + core::mem::size_of::<#types>() )*,
                "Cells: padding between the fields",
            );
        };
        // SAFETY: every field is `Cells` and no byte lies outside them (checked above).
        unsafe impl #with crate::pointers::Cells for #name #of #clause {
            fn cell_at(offset: usize) -> crate::pointers::CellAt {
                // The fields' first bytes, rising (`repr(C)`): the field holding `offset` is
                // the last starting at or before it (one of no size never holds it).
                let starts = [#( core::mem::offset_of!(Self, #names) ),*];
                let field = starts.partition_point(|&start| start <= offset).wrapping_sub(1);
                match field {
                    #( #indices => <#types as crate::pointers::Cells>::cell_at(
                        offset - starts[#indices],
                    )
                    .after(starts[#indices]), )*
                    _ => panic!("Cells: byte {offset} outside the record"),
                }
            }
        }
    }
    .into()
}
