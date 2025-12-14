use proc_macro::TokenStream;
use quote::*;
use syn::*;

mod multi_param;

/// Annotates a struct as a Dantelion2 singleton to be looked up using a single
/// string argument.
///
/// This is only guaranteed to make the struct work with the
/// `fromsoftware_shared::singleton::get_instance` function. Any other added
/// functionality is considered an implementation detail and shouldn't be relied
/// upon.
#[proc_macro_attribute]
pub fn singleton(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct: ItemStruct = parse_macro_input!(input as ItemStruct);
    let input_struct_ident = input_struct.ident.clone();
    let dlrf_name = parse_macro_input!(args as LitStr).value();

    TokenStream::from(quote! {
        #input_struct

        impl ::from_singleton::FromSingleton for #input_struct_ident {
            fn name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(#dlrf_name)
            }
        }
    })
}

/// Annotates a trait to automatically generate getters and setters that forward
/// to methods of the same name in various structs.
///
/// This is used to create traits that encapsulate state that's shared across
/// multiple parameter definitions.
///
/// This trait takes as arguments the names of various structs for which it
/// should automatically generate an implementation. It should annotate a trait
/// that contains a `fields!` macro, using the same named field syntax that a
/// struct uses. For each field, a getter and setter is generated both in the
/// trait and in its implementation for each struct.
///
/// ## Casts
///
/// The trait may include functions annotated with `#[multi_param(cast)]`. These
/// functions must return an [Option] of a reference to one of the parameter
/// struct types. The macro will generate implementations for these functions,
/// as well as `..._mut()` implementations, that downcast to the given return
/// type. For example:
///
/// ```rs
/// #[multi_param(cast)]
/// fn as_weapon(&self) -> Option<&EQUIP_PARAM_WEAPON_ST>;
/// ```
#[proc_macro_attribute]
pub fn multi_param(args: TokenStream, input: TokenStream) -> TokenStream {
    match multi_param::multi_param_helper(args, input) {
        Ok(stream) => stream,
        Err(err) => err.into_compile_error().into(),
    }
}
