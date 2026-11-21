//! `#[derive(Cells)]` and `#[derive(ByteView)]` for `rc2k_decomp::cells`, and `#[derive(ZeroCells)]`.
//! `Cells` makes a record of cells copy, merge and be visited field by field, in any layout.
//! `ByteView` adds the byte view of a `#[repr(C)]` or `#[repr(transparent)]` record: the runs of
//! cells found by byte offset, and the debug pages' field paths. Both derives check at compile
//! time, so the promises the traits ask for rest on the primitive cells alone.

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as Tokens, TokenTree};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// `tokens` with every identifier `from` replaced by `to` (a field's type in the other kind).
fn rename(
    tokens: proc_macro2::TokenStream,
    from: &syn::Ident,
    to: &syn::Ident,
) -> proc_macro2::TokenStream {
    tokens
        .into_iter()
        .map(|t| match t {
            TokenTree::Ident(i) if i == *from => TokenTree::Ident(to.clone()),
            TokenTree::Group(g) => {
                let mut group =
                    proc_macro2::Group::new(g.delimiter(), rename(g.stream(), from, to));
                group.set_span(g.span());
                TokenTree::Group(group)
            }
            other => other,
        })
        .collect()
}

/// `input` parsed as the item a derive is on, or the compile error that says it is not one.
fn parse_item(input: Tokens) -> Result<DeriveInput, Tokens> {
    syn::parse2(input).map_err(|e| e.to_compile_error())
}

/// The fields of a struct as their types and the tokens naming them (`name` or the index).
fn struct_fields(input: &DeriveInput, what: &str) -> Result<(Vec<syn::Type>, Vec<Tokens>), Tokens> {
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.named.iter().collect::<Vec<_>>(),
            Fields::Unnamed(f) => f.unnamed.iter().collect(),
            Fields::Unit => Vec::new(),
        },
        _ => {
            return Err(
                syn::Error::new_spanned(&input.ident, format!("{what}: only a struct"))
                    .to_compile_error(),
            );
        }
    };
    let types = fields.iter().map(|f| f.ty.clone()).collect();
    let names = fields
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
    Ok((types, names))
}

/// `#[derive(Cells)]` for `crate::memory::cells::Cells`: a record is made of cells when each of its
/// fields is, in any layout. The derived `copy_from` copies field by field, in the fields'
/// order, `merge_changes` merges field by field, and `visit` goes through the fields by name.
/// A field named `inline` adds nothing to the debug pages' field paths: its own fields read as
/// the record's (a part held in a `GameThread` among shared cells). The byte view of a record
/// (its fixed layout) is `#[derive(ByteView)]`; test builds add it to every `#[repr(C)]`
/// record.
#[proc_macro_derive(Cells)]
pub fn derive_cells(input: TokenStream) -> TokenStream {
    expand_cells(input.into()).into()
}

/// The code `#[derive(Cells)]` writes for the item `input`, or the compile error.
fn expand_cells(input: Tokens) -> Tokens {
    let input = match parse_item(input) {
        Ok(input) => input,
        Err(error) => return error,
    };
    let name = &input.ident;
    let (with, of, clause) = input.generics.split_for_impl();
    let (types, names) = match struct_fields(&input, "Cells") {
        Ok(fields) => fields,
        Err(error) => return error,
    };
    // The checks: for a record with no generics every field is `Cells` at compile time; for a
    // generic one (a record kept in two kinds of cells, `session::CellKind`) every field is
    // bound to `Cells` on the impl.
    let generic = !input.generics.params.is_empty();
    let (checks, clause) = if generic {
        let mut clause = clause.cloned().unwrap_or_else(|| syn::parse_quote!(where));
        for ty in &types {
            clause
                .predicates
                .push(syn::parse_quote!(#ty: crate::memory::cells::Cells));
        }
        (quote! {}, Some(clause))
    } else {
        let checks = quote! {
            const _: () = {
                const fn is_cells<T: crate::memory::cells::Cells + ?Sized>() {}
                #( is_cells::<#types>(); )*
            };
        };
        (checks, clause.cloned())
    };
    // `visit`: each field entered by its name, but for `inline`.
    let visits: Vec<_> = names
        .iter()
        .map(|n| {
            let text = n.to_string();
            if text == "inline" {
                quote! { crate::memory::cells::Cells::visit(&self.#n, visitor); }
            } else {
                quote! {
                    visitor.enter(#text);
                    crate::memory::cells::Cells::visit(&self.#n, visitor);
                    visitor.leave();
                }
            }
        })
        .collect();
    // `SameShape`: a record over a kind `K` (one type parameter) copied and merged field by
    // field into the same record over another kind `K2`, with no byte layout.
    let same_shape = match input.generics.params.first() {
        Some(syn::GenericParam::Type(kind)) if input.generics.params.len() == 1 => {
            let k = &kind.ident;
            let k2 = syn::Ident::new("K2__", k.span());
            let mut clause2 = clause.clone().unwrap_or_else(|| syn::parse_quote!(where));
            let bounds = kind.bounds.clone();
            for ty in &types {
                let other = rename(quote!(#ty), k, &k2);
                clause2
                    .predicates
                    .push(syn::parse_quote!(#ty: crate::memory::cells::SameShape<#other>));
            }
            let mut params = input.generics.clone();
            params.params.push(syn::parse_quote!(#k2: #bounds));
            let (with2, _, _) = params.split_for_impl();
            quote! {
                impl #with2 crate::memory::cells::SameShape<#name<#k2>> for #name<#k> #clause2 {
                    fn copy_cells_to(&self, to: &#name<#k2>) {
                        #( crate::memory::cells::SameShape::copy_cells_to(&self.#names, &to.#names); )*
                    }

                    fn merge_cells_to(&self, start: &Self, to: &#name<#k2>) {
                        #( crate::memory::cells::SameShape::merge_cells_to(&self.#names, &start.#names, &to.#names); )*
                    }
                }
            }
        }
        _ => quote! {},
    };
    // Test builds: the byte view of every `#[repr(C)]` record too (see `derive_byte_view`).
    let byte_view = if is_repr_c(&input) {
        match byte_view_impl(&input, quote!(#[cfg(test)])) {
            Ok(tokens) => tokens,
            Err(error) => return error,
        }
    } else {
        quote! {}
    };
    quote! {
        #checks
        #same_shape
        #byte_view
        // SAFETY: every field is `Cells`, so every byte of the record is in a cell and all
        // zero bytes are a valid value.
        unsafe impl #with crate::memory::cells::Cells for #name #of #clause {
            fn copy_from(&self, other: &Self) {
                #( crate::memory::cells::Cells::copy_from(&self.#names, &other.#names); )*
            }

            fn merge_changes(&self, now: &Self, start: &Self) {
                #( crate::memory::cells::Cells::merge_changes(&self.#names, &now.#names, &start.#names); )*
            }

            fn visit(&self, visitor: &mut dyn crate::memory::cells::CellVisitor) {
                #( #visits )*
            }
        }
    }
}

/// Whether the record is `#[repr(C)]` or `#[repr(transparent)]`.
fn is_repr_c(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|a| {
        let text = quote!(#a).to_string();
        a.path().is_ident("repr") && (text.contains('C') || text.contains("transparent"))
    })
}

/// `#[derive(ByteView)]` for `crate::memory::cells::ByteView`: a `#[repr(C)]` or `#[repr(transparent)]`
/// record of `ByteView` fields with no byte outside them. Both are checked at compile time, so
/// the promise the trait asks for rests on the primitive cells alone. The derived `cell_run`
/// finds a byte's field by its offset and asks the field's type, down to a run of primitive
/// cells; `field_path` (tests and the `remote` build) names it.
///
/// Test builds get the impl from `#[derive(Cells)]` on every `#[repr(C)]` record (the tests'
/// registry walks all of the game's stores by byte), so this derive adds it to the other
/// builds only.
#[proc_macro_derive(ByteView)]
pub fn derive_byte_view(input: TokenStream) -> TokenStream {
    expand_byte_view(input.into()).into()
}

/// The code `#[derive(ByteView)]` writes for the item `input`, or the compile error.
fn expand_byte_view(input: Tokens) -> Tokens {
    let input = match parse_item(input) {
        Ok(input) => input,
        Err(error) => return error,
    };
    if !is_repr_c(&input) {
        return syn::Error::new_spanned(
            &input.ident,
            "ByteView: needs #[repr(C)] or #[repr(transparent)]",
        )
        .to_compile_error();
    }
    byte_view_impl(&input, quote!(#[cfg(not(test))])).unwrap_or_else(|error| error)
}

/// The `ByteView` impl and its checks of `input`, each item carrying `cfg`.
fn byte_view_impl(input: &DeriveInput, cfg: Tokens) -> Result<Tokens, Tokens> {
    let name = &input.ident;
    let (with, of, clause) = input.generics.split_for_impl();
    let (types, names) = struct_fields(input, "ByteView")?;
    let indices: Vec<_> = (0..names.len()).collect();
    let count = names.len();
    // A table built once at compile time where the record has no generics (an item inside the
    // function cannot name them); with generics, the array is built per call.
    let starts = if input.generics.params.is_empty() {
        quote! {
            const STARTS: [usize; #count] = [#( ::core::mem::offset_of!(#name, #names) ),*];
            let starts = &STARTS;
        }
    } else {
        quote! { let starts = [#( ::core::mem::offset_of!(Self, #names) ),*]; }
    };
    // `field_path` (tests only): the path of the primitive cell holding a byte, from the record's
    // field names, for the debug pages' tables.
    let path_names: Vec<_> = names
        .iter()
        .map(|n| {
            if n.to_string() == "inline" {
                String::new()
            } else {
                format!(".{n}")
            }
        })
        .collect();
    // The checks: at compile time for a record with no generics; for a generic one (a record
    // kept in two kinds of cells, `session::CellKind`) every field bound to `ByteView` on the
    // impl and the padding asserted per kind, where the impl is first used.
    let generic = !input.generics.params.is_empty();
    let (checks, clause, checked) = if generic {
        let mut clause = clause.cloned().unwrap_or_else(|| syn::parse_quote!(where));
        for ty in &types {
            clause
                .predicates
                .push(syn::parse_quote!(#ty: crate::memory::cells::ByteView));
        }
        let checks = quote! {
            #cfg
            impl #with #name #of #clause {
                const BYTE_VIEW_NO_PADDING: () = assert!(
                    ::core::mem::size_of::<Self>() == 0 #( + ::core::mem::size_of::<#types>() )*,
                    "ByteView: padding between the fields",
                );
            }
        };
        (
            checks,
            Some(clause),
            quote! { let () = Self::BYTE_VIEW_NO_PADDING; },
        )
    } else {
        let checks = quote! {
            #cfg
            const _: () = {
                const fn is_byte_view<T: crate::memory::cells::ByteView + ?Sized>() {}
                #( is_byte_view::<#types>(); )*
                assert!(
                    ::core::mem::size_of::<#name #of>() == 0 #( + ::core::mem::size_of::<#types>() )*,
                    "ByteView: padding between the fields",
                );
            };
        };
        (checks, clause.cloned(), quote! {})
    };
    Ok(quote! {
        #checks
        #cfg
        // SAFETY: `#[repr(C)]` over `ByteView` fields and no byte outside them (checked
        // above), so any bytes make a value and every byte is in a cell.
        unsafe impl #with crate::memory::cells::ByteView for #name #of #clause {
            #[cfg(test)]
            fn field_path(&self, offset: usize, path: &mut String) -> &'static str {
                #starts
                let field = starts.partition_point(|&start| start <= offset).wrapping_sub(1);
                match field {
                    #( #indices => {
                        path.push_str(#path_names);
                        crate::memory::cells::ByteView::field_path(&self.#names, offset - starts[#indices], path)
                    } )*
                    _ => panic!("ByteView: byte {offset} outside the record"),
                }
            }

            fn cell_run(&self, offset: usize) -> (usize, crate::memory::cells::CellRun<'_>) {
                #checked
                #starts
                let field = starts.partition_point(|&start| start <= offset).wrapping_sub(1);
                match field {
                    #( #indices => {
                        let (start, run) = crate::memory::cells::ByteView::cell_run(
                            &self.#names,
                            offset - starts[#indices],
                        );
                        (starts[#indices] + start, run)
                    } )*
                    _ => panic!("ByteView: byte {offset} outside the record"),
                }
            }
        }
    })
}

/// `#[derive(ZeroCells)]` for `crate::memory::cells::ZeroCells`: the record whose every field is
/// zero, as a constant, so a store's `new()` can name only the fields that start non-zero
/// (`Record { count: Dword::new(3), ..ZeroCells::ZERO }`). Every field's type is `ZeroCells`.
#[proc_macro_derive(ZeroCells)]
pub fn derive_zero_cells(input: TokenStream) -> TokenStream {
    expand_zero_cells(input.into()).into()
}

/// The code `#[derive(ZeroCells)]` writes for the item `input`, or the compile error.
fn expand_zero_cells(input: Tokens) -> Tokens {
    let input = match parse_item(input) {
        Ok(input) => input,
        Err(error) => return error,
    };
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(name, "ZeroCells: no generics").to_compile_error();
    }
    let Data::Struct(s) = &input.data else {
        return syn::Error::new_spanned(name, "ZeroCells: only a struct").to_compile_error();
    };
    let init = match &s.fields {
        Fields::Named(f) => {
            let names = f.named.iter().map(|f| &f.ident);
            let types = f.named.iter().map(|f| &f.ty);
            quote! { #name { #( #names: <#types as crate::memory::cells::ZeroCells>::ZERO ),* } }
        }
        Fields::Unnamed(f) => {
            let types = f.unnamed.iter().map(|f| &f.ty);
            quote! { #name( #( <#types as crate::memory::cells::ZeroCells>::ZERO ),* ) }
        }
        Fields::Unit => quote! { #name },
    };
    quote! {
        impl crate::memory::cells::ZeroCells for #name {
            const ZERO: Self = #init;
        }
    }
}

#[cfg(test)]
mod tests {
    //! The derives' expansions as text (no compile-fail harness, so no dependency beyond the
    //! macros' own): what is rejected gives a `compile_error!` with its reason, and what is
    //! accepted carries the checks the promises rest on.

    use super::*;

    fn text(tokens: Tokens) -> String {
        tokens.to_string()
    }

    fn rejected(tokens: Tokens, reason: &str) {
        let out = text(tokens);
        assert!(out.contains("compile_error"), "no error in {out}");
        assert!(out.contains(reason), "{reason} not in {out}");
    }

    #[test]
    fn a_derive_on_something_that_is_not_a_struct_is_an_error() {
        let item = quote! { enum Kind { A, B } };
        rejected(expand_cells(item.clone()), "Cells: only a struct");
        rejected(expand_zero_cells(item), "ZeroCells: only a struct");
        let item = quote! { #[repr(C)] union Either { a: u32, b: u32 } };
        rejected(expand_cells(item.clone()), "Cells: only a struct");
        rejected(expand_byte_view(item), "ByteView: only a struct");
    }

    #[test]
    fn byte_view_needs_a_repr_that_fixes_the_layout() {
        rejected(
            expand_byte_view(quote! { struct Loose { a: Dword } }),
            "needs #[repr(C)] or #[repr(transparent)]",
        );
        rejected(
            expand_byte_view(quote! { #[repr(u8)] struct Narrow { a: Dword } }),
            "needs #[repr(C)] or #[repr(transparent)]",
        );
        assert!(
            !text(expand_byte_view(
                quote! { #[repr(C)] struct Fixed { a: Dword } }
            ))
            .contains("compile_error")
        );
        assert!(
            !text(expand_byte_view(
                quote! { #[repr(transparent)] struct One(Dword); }
            ))
            .contains("compile_error")
        );
    }

    #[test]
    fn zero_cells_takes_no_generics() {
        rejected(
            expand_zero_cells(quote! { struct Over<K> { a: K } }),
            "ZeroCells: no generics",
        );
    }

    #[test]
    fn input_that_is_not_an_item_is_an_error() {
        rejected(expand_cells(quote! { fn not_an_item() {} }), "expected");
        rejected(expand_byte_view(quote! { 1 + 2 }), "expected");
    }

    #[test]
    fn a_record_without_generics_has_every_field_checked_at_compile_time() {
        let out = text(expand_cells(quote! { struct Pair { a: Dword, b: Word } }));
        assert!(out.contains("is_cells :: < Dword >"));
        assert!(out.contains("is_cells :: < Word >"));
        // field by field, in the fields' order
        let (a, b) = (out.find("self . a").unwrap(), out.find("self . b").unwrap());
        assert!(a < b);
    }

    #[test]
    fn a_generic_record_bounds_its_fields_on_the_impl_instead() {
        let out = text(expand_cells(quote! { struct Held<K: Kind> { a: Cell<K> } }));
        assert!(out.contains("Cell < K > : crate :: memory :: cells :: Cells"));
        assert!(!out.contains("is_cells"));
        assert!(
            out.contains("SameShape"),
            "a record over one kind has its other kind"
        );
    }

    #[test]
    fn the_byte_view_checks_that_no_byte_is_outside_the_fields() {
        let out = text(expand_byte_view(
            quote! { #[repr(C)] struct Gap { a: u8, b: u32 } },
        ));
        assert!(out.contains("padding between the fields"));
        assert!(out.contains("is_byte_view :: < u8 >"));
        let generic = text(expand_byte_view(
            quote! { #[repr(C)] struct Gen<K> { a: Cell<K> } },
        ));
        assert!(generic.contains("BYTE_VIEW_NO_PADDING"));
    }

    #[test]
    fn the_generated_byte_view_refuses_a_byte_past_the_record() {
        // The two `panic!` arms are in the generated code: reached by the users' tests (a
        // byte outside every field); here the text shows they are there.
        let out = text(expand_byte_view(
            quote! { #[repr(C)] struct One { a: Dword } },
        ));
        assert_eq!(out.matches("outside the record").count(), 2);
    }

    #[test]
    fn a_field_named_inline_adds_nothing_to_the_path() {
        let out = text(expand_cells(
            quote! { struct Part { inline: Dword, other: Dword } },
        ));
        assert!(out.contains("enter (\"other\")"));
        assert!(!out.contains("enter (\"inline\")"));
        let view = text(expand_byte_view(
            quote! { #[repr(C)] struct Part { inline: Dword, other: Dword } },
        ));
        assert!(view.contains("push_str (\"\")"));
        assert!(view.contains("push_str (\".other\")"));
    }

    #[test]
    fn zero_cells_gives_every_field_its_zero() {
        let out = text(expand_zero_cells(
            quote! { struct Pair { a: Dword, b: Word } },
        ));
        assert!(out.contains("a : < Dword as crate :: memory :: cells :: ZeroCells > :: ZERO"));
        let tuple = text(expand_zero_cells(quote! { struct Wrap(Dword); }));
        assert!(tuple.contains("Wrap (< Dword as"));
        let unit = text(expand_zero_cells(quote! { struct Nothing; }));
        assert!(unit.contains("const ZERO : Self = Nothing"));
    }
}
