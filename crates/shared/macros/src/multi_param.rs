use std::collections::{HashMap, hash_map::Entry};
use std::iter::{IntoIterator, Iterator};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::*;
use syn::*;
use syn::{
    meta::ParseNestedMeta,
    parse::{ParseBuffer, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// A helper for [multi_param] that returns a [syn::Result].
pub fn multi_param_helper(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut input_trait: ItemTrait = syn::parse(input)?;
    let structs = Punctuated::<TypePath, Token![,]>::parse_terminated
        .parse(args)?
        .into_iter()
        .collect::<Vec<_>>();

    let fields = extract_fields(&mut input_trait, &structs)?;
    for field in &fields {
        let ident = &field.ident;
        let set_ident = format_ident!("set_{}", field.ident);
        let ty = &field.ty;
        input_trait.items.push(syn::parse2(quote_spanned! {
            field.span => fn #ident(&self) -> #ty;
        })?);
        input_trait.items.push(syn::parse2(quote_spanned! {
            field.span => fn #set_ident(&mut self, value: #ty);
        })?);
    }

    let methods = extract_methods(&mut input_trait)?;
    for method in &methods {
        let ident = &method.ident;
        let ident_mut = format_ident!("{}_mut", ident);
        let ty = &method.ty;
        input_trait.items.push(syn::parse2(quote_spanned! {
            method.span => fn #ident(&self) -> Option<&#ty>;
        })?);
        input_trait.items.push(syn::parse2(quote_spanned! {
            method.span => fn #ident_mut(&mut self) -> Option<&mut #ty>;
        })?);
    }

    let impls = structs
        .into_iter()
        .map(|struct_| generate_impl(&input_trait.ident, struct_, &fields, &methods))
        .collect::<Result<Vec<_>>>()?;

    Ok(TokenStream::from(quote! {
        #input_trait

        #(#impls)*
    }))
}

/// A field declared in the `fields!` macro that should be dispatched to each
/// parameter struct.
struct MultiParamField {
    /// The default field name.
    ident: Ident,

    /// The field's type.
    ty: Type,

    /// The span at which the field was declared.
    span: Span,

    /// Specialized field names to use for particular structs.
    renames: HashMap<TypePath, Ident>,
}

/// Returns all fields in [trait_] declared with `fields!`.
fn extract_fields(trait_: &mut ItemTrait, structs: &[TypePath]) -> Result<Vec<MultiParamField>> {
    trait_
        .items
        .extract_if(.., |item| match item {
            TraitItem::Macro(mac) => mac.mac.path.is_ident("fields"),
            _ => false,
        })
        .filter_map(|item| match item {
            TraitItem::Macro(mac) => Some(mac),
            _ => None,
        })
        .flat_map(|mac| {
            let parser = |input: &ParseBuffer<'_>| {
                Punctuated::<Field, Token![,]>::parse_terminated_with(input, Field::parse_named)
            };
            let fields = match parser.parse2(mac.mac.tokens) {
                Ok(fields) => fields,
                Err(err) => return vec![Err(err)],
            };

            fields
                .into_iter()
                .map(|mut field| {
                    let attributes = extract_field_attributes(&mut field)?;
                    let mut renames = HashMap::new();
                    for attr in attributes {
                        match attr {
                            FieldAttribute::Rename(param, name) => {
                                if !structs.contains(&param) {
                                    return Err(Error::new(
                                        param.span(),
                                        "this isn't one of the multi_param() arguments",
                                    ));
                                }

                                match renames.entry(param) {
                                    Entry::Occupied(o) => {
                                        return Err(Error::new(o.key().span(), "duplicate param"));
                                    }
                                    Entry::Vacant(v) => v.insert(name),
                                }
                            }
                        };
                    }

                    if !field.attrs.is_empty() {
                        Err(Error::new(
                            field.attrs[0].span(),
                            "multi_param fields may only have #[multi_param(...)] attributes",
                        ))
                    } else if field.vis != Visibility::Inherited {
                        Err(Error::new(
                            field.span(),
                            "multi_param fields must have default visibility",
                        ))
                    } else {
                        let span = field.span();
                        Ok(MultiParamField {
                            ident: field.ident.unwrap(),
                            ty: field.ty,
                            span,
                            renames,
                        })
                    }
                })
                .collect()
        })
        .collect()
}

/// A `multi_param()` attribute on a field.
enum FieldAttribute {
    /// `rename(struct = ..., name = ...)`
    Rename(TypePath, Ident),
}

/// Removes all `#[multi_param(...)]` attributes from [field] and returns them
/// as [FieldAttribute]s.
fn extract_field_attributes(field: &mut Field) -> Result<Vec<FieldAttribute>> {
    field
        .attrs
        .extract_if(.., |attr| attr.path().is_ident("multi_param"))
        .flat_map(|attr| {
            let mut attributes = Vec::new();
            if let Err(err) = attr.parse_nested_meta(|meta| {
                attributes.push(Ok(parse_field_attribute(meta)?));
                Ok(())
            }) {
                return vec![Err(err)];
            }
            attributes
        })
        .collect()
}

/// Parses a single nested meta item inside a `#[multi_param(...)]` attribute on
/// a field in `fields!`.
fn parse_field_attribute(meta: ParseNestedMeta<'_>) -> Result<FieldAttribute> {
    if !meta.path.is_ident("rename") {
        return Err(meta.error("unrecognized attribute"));
    }

    let mut param: Option<TypePath> = None;
    let mut name: Option<Ident> = None;
    meta.parse_nested_meta(|arg| {
        if arg.path.is_ident("param") {
            param = Some(arg.value()?.parse()?);
            Ok(())
        } else if arg.path.is_ident("name") {
            name = Some(arg.value()?.parse::<LitStr>()?.parse()?);
            Ok(())
        } else {
            Err(arg.error("unrecognized argument"))
        }
    })?;

    match (param, name) {
        (Some(param), Some(name)) => Ok(FieldAttribute::Rename(param, name)),
        (None, _) => Err(meta.error("missing argument \"param\"")),
        (_, None) => Err(meta.error("missing argument \"name\"")),
    }
}

/// A method that casts the parameter to one of its specific types.
struct MultiParamCast {
    /// The method name.
    ident: Ident,

    /// The type to cast this to.
    ty: TypePath,

    /// The span at which the method was declared.
    span: Span,
}

/// Returns all methods in [trait_] with specific `multi_param` semantics.
fn extract_methods(trait_: &mut ItemTrait) -> Result<Vec<MultiParamCast>> {
    trait_
        .items
        .extract_if(.., |item| match item {
            TraitItem::Fn(f) => f.attrs.iter().any(|a| a.path().is_ident("multi_param")),
            _ => false,
        })
        .filter_map(|item| match item {
            TraitItem::Fn(f) => Some(f),
            _ => None,
        })
        .map(|mut method| {
            // Right now, this is only going to be cast.
            extract_method_attributes(&mut method)?;
            if !method.attrs.is_empty() {
                return Err(Error::new(
                    method.attrs[0].span(),
                    "casts may only have #[multi_param(...)] attributes",
                ));
            }

            let span = method.span();
            let sig_span = method.sig.span();
            expect_none(method.sig.constness, "cast")?;
            expect_none(method.sig.asyncness, "cast")?;
            expect_none(method.sig.unsafety, "cast")?;
            expect_none(method.sig.abi, "cast")?;
            expect_empty(method.sig.generics.params, "cast")?;
            expect_none(method.sig.generics.where_clause, "cast")?;
            if method.sig.inputs.len() > 1 {
                return Err(Error::new(
                    method.sig.inputs.span(),
                    "casts can't have parameters",
                ));
            } else {
                match &method.sig.inputs[0] {
                    FnArg::Receiver(Receiver {
                        reference: Some((_, None)),
                        mutability: None,
                        colon_token: None,
                        ..
                    }) => {}
                    arg => return Err(Error::new(arg.span(), "expected &self")),
                }
            }
            expect_none(method.sig.variadic, "cast")?;
            expect_none(method.default, "cast")?;

            let ReturnType::Type(_, return_type) = method.sig.output else {
                return Err(Error::new(sig_span, "cast must have a return type"));
            };
            let Some(ty) = parse_cast_type(*return_type) else {
                return Err(Error::new(sig_span, "cast must return Option<&Type>"));
            };

            Ok(MultiParamCast {
                ident: method.sig.ident,
                ty,
                span,
            })
        })
        .collect()
}

fn parse_cast_type(ty: Type) -> Option<TypePath> {
    if let Type::Path(mut path) = ty
        && path.qself.is_none()
        && path.path.segments.len() == 1
        && let Some(pair) = path.path.segments.pop()
        && let PathSegment {
            ident,
            arguments: PathArguments::AngleBracketed(mut args),
        } = pair.into_value()
        && ident == "Option"
        && args.args.len() == 1
        && let Some(pair) = args.args.pop()
        && let GenericArgument::Type(Type::Reference(TypeReference {
            lifetime: None,
            mutability: None,
            elem,
            ..
        })) = pair.into_value()
        && let Type::Path(path) = *elem
    {
        Some(path)
    } else {
        None
    }
}

/// A `multi_param()` attribute on a method.
enum MethodAttribute {
    /// `cast`, with no arguments
    Cast,
}

/// Removes all `#[multi_param(...)]` attributes from [f] and returns them
/// as [MethodAttribute]s.
fn extract_method_attributes(f: &mut TraitItemFn) -> Result<Vec<MethodAttribute>> {
    f.attrs
        .extract_if(.., |attr| attr.path().is_ident("multi_param"))
        .flat_map(|attr| {
            let mut attributes = Vec::new();
            if let Err(err) = attr.parse_nested_meta(|meta| {
                attributes.push(Ok(parse_method_attribute(meta)?));
                Ok(())
            }) {
                return vec![Err(err)];
            }
            attributes
        })
        .collect()
}

/// Parses a single nested meta item inside a `#[multi_param(...)]` attribute on
/// a fn in a multi_param trait.
fn parse_method_attribute(meta: ParseNestedMeta<'_>) -> Result<MethodAttribute> {
    if !meta.path.is_ident("cast") {
        Err(meta.error("unrecognized attribute"))
    } else if !meta.input.is_empty() {
        Err(meta.error("expected no arguments"))
    } else {
        Ok(MethodAttribute::Cast)
    }
}

/// Generates an implementation of [trait_] for [target] which forwards getters
/// and setters for all fields in [fields] to methods of the same name.
fn generate_impl<'a>(
    trait_: &Ident,
    target: TypePath,
    fields: impl IntoIterator<Item = &'a MultiParamField>,
    methods: impl IntoIterator<Item = &'a MultiParamCast>,
) -> Result<ItemImpl> {
    let mut result: ItemImpl = syn::parse2(quote! {
        impl #trait_ for #target {}
    })?;

    for MultiParamField {
        ident,
        ty,
        span,
        renames,
    } in fields
    {
        let target_ident = renames.get(&target).unwrap_or(ident);

        result.items.push(syn::parse2(quote_spanned! { *span =>
            fn #ident(&self) -> #ty {
                #target::#target_ident(self)
            }
        })?);

        let set_ident = format_ident!("set_{}", ident);
        let set_target_ident = format_ident!("set_{}", target_ident);
        result.items.push(syn::parse2(quote_spanned! { *span =>
            fn #set_ident(&mut self, value: #ty) {
                #target::#set_target_ident(self, value)
            }
        })?);
    }

    for MultiParamCast { ident, ty, span } in methods {
        let body = if *ty == target {
            quote! { Some(self) }
        } else {
            quote! { None }
        };
        result.items.push(syn::parse2(quote_spanned! { *span =>
            fn #ident(&self) -> Option<&#ty> {
                #body
            }
        })?);

        let ident_mut = format_ident!("{}_mut", ident);
        result.items.push(syn::parse2(quote_spanned! { *span =>
            fn #ident_mut(&mut self) -> Option<&mut #ty> {
                #body
            }
        })?);
    }

    Ok(result)
}

/// Returns an error if [value] isn't `None`.
fn expect_none<T>(value: Option<T>, context: &str) -> Result<()>
where
    T: Spanned,
{
    if let Some(value) = value {
        Err(Error::new(
            value.span(),
            format!("not allowed in {}", context),
        ))
    } else {
        Ok(())
    }
}

/// Returns an error if [value] isn't empty.
fn expect_empty<T>(value: T, context: &str) -> Result<()>
where
    T: IntoIterator + Spanned,
{
    let span = value.span();
    if let Some(_) = value.into_iter().next() {
        Err(Error::new(span, format!("not allowed in {}", context)))
    } else {
        Ok(())
    }
}
