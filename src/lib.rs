#![feature(external_doc)]
#![doc(include = "../readme.md")]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse, ItemStruct, Fields, ExprRange, RangeLimits};
use quote::quote;

/// A procedural macro that allows user to define ranged integer types.
///
/// It can only be applied to a tuple struct type with a single field.
/// The type of this field is not checked, but normally it can only be an integer type.
/// The content within `#[ranged(...)]` should be a closed range.
#[proc_macro_attribute]
pub fn ranged(attr: TokenStream, item: TokenStream) -> TokenStream {
  fn work(attr: TokenStream, item: TokenStream) -> Option<TokenStream> {
    let s = parse::<ItemStruct>(item).ok()?;
    if let Fields::Unnamed(fs) = &s.fields {
      let mut it = fs.unnamed.iter();
      let f = it.next()?;
      if it.next().is_some() { return None; }
      let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();
      let (s_ident, s_attr, s_vis) = (&s.ident, &s.attrs, &s.vis);
      let f_ty = &f.ty;
      if let Ok(ExprRange { from, to, limits: RangeLimits::Closed(_), .. }) = parse::<ExprRange>(attr) {
        let (from, to) = (from.as_ref(), to.as_ref());
        let lb_ck = from.map(|x| quote!(if v < #x { return None; }));
        let ub_ck = to.map(|x| quote!(if v > #x { return None; }));
        let (lb_layout, ub_layout) = if cfg!(feature = "rustc-layout") {
          (from.map(|x| quote!(#[rustc_layout_scalar_valid_range_start(#x)])),
           to.map(|x| quote!(#[rustc_layout_scalar_valid_range_end(#x)])))
        } else { (None, None) };
        let (lb_assume, ub_assume) = if cfg!(feature = "assume-hint") {
          (from.map(|x| quote!(unsafe { ::core::intrinsics::assume(self.0 >= #x) })),
           to.map(|x| quote!(unsafe { ::core::intrinsics::assume(self.0 <= #x) })))
        } else { (None, None) };
        Some(quote! {
          #lb_layout
          #ub_layout
          #(#s_attr)*
          #s_vis struct #s_ident(#f_ty);

          impl #impl_generics #s_ident #ty_generics #where_clause {
            pub fn new(v: #f_ty) -> Option<Self> {
              #lb_ck
              #ub_ck
              unsafe { Some(Self(v)) }
            }

            pub unsafe fn new_unchecked(v: #f_ty) -> Self {
              Self(v)
            }

            pub fn get(self) -> #f_ty {
              #lb_assume
              #ub_assume
              self.0
            }
          }
        }.into())
      } else { panic!("`range` expect attribute to be an closed range (like `1..=2`, `1..=`, `..=2`)"); }
    } else { None }
  }
  work(attr, item).unwrap_or_else(|| panic!("`ranged` can only be applied to a tuple struct with a single field"))
}