use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Field, Fields, Ident,
    LitStr, Path, Type,
};

#[proc_macro_derive(Fields, attributes(sqlx, sqlx_paginated))]
pub fn derive_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_fields(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Paginated, attributes(sqlx_paginated))]
pub fn derive_paginated(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_paginated(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_fields(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = input.ident;
    let vis = input.vis;
    let enum_ident = format_ident!("{}Field", struct_ident);
    let rename_all = parse_rename_all(&input.attrs)?;
    let container = ContainerAttrs::from_attrs(&input.attrs)?;
    let crate_path = &container.crate_path;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(syn::Error::new(
                    struct_ident.span(),
                    "Fields only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                struct_ident.span(),
                "Fields only supports structs",
            ))
        }
    };

    let mut variants = Vec::new();
    let mut as_str_arms = Vec::new();
    let mut contains_arms = Vec::new();

    for field in fields {
        match parse_field(&field, rename_all.as_deref())? {
            FieldBehavior::Skip => {}
            FieldBehavior::Direct {
                variant_ident,
                output_name,
            } => {
                variants.push(quote! { #variant_ident });
                as_str_arms.push(quote! {
                    Self::#variant_ident => #output_name
                });
                contains_arms.push(quote! {
                    #output_name => true,
                });
            }
            FieldBehavior::Flatten { variant_ident, ty } => {
                let child_enum_ident = format_ident!("{}Field", type_to_ident(&ty)?);
                variants.push(quote! { #variant_ident(#child_enum_ident) });
                as_str_arms.push(quote! {
                    Self::#variant_ident(inner) => inner.as_str()
                });
                contains_arms.push(quote! {
                    _ if <#child_enum_ident as #crate_path::FieldEnum>::contains(value) => true,
                });
            }
        }
    }

    Ok(quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #vis enum #enum_ident {
            #(#variants),*
        }

        impl #enum_ident {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms),*
                }
            }
        }

        impl #crate_path::FieldEnum for #enum_ident {
            fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms),*
                }
            }

            fn contains<S: AsRef<str>>(s: S) -> bool {
                let value = s.as_ref();
                match value {
                    #(#contains_arms)*
                    _ => false,
                }
            }
        }

        impl ::std::convert::From<#enum_ident> for ::std::string::String {
            fn from(value: #enum_ident) -> Self {
                value.as_str().to_owned()
            }
        }
    })
}

fn expand_paginated(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = input.ident;
    let enum_ident = format_ident!("{}Field", struct_ident);
    let generics = input.generics;
    let container = ContainerAttrs::from_attrs(&input.attrs)?;
    let crate_path = &container.crate_path;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::PaginatedInfo for #struct_ident #ty_generics #where_clause {
            type Fields = #enum_ident;
        }
    })
}

struct ContainerAttrs {
    crate_path: Path,
}

impl ContainerAttrs {
    fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut crate_path = None;

        for attr in attrs {
            if !attr.path().is_ident("sqlx_paginated") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    let value: LitStr = meta.value()?.parse()?;
                    if crate_path.is_some() {
                        return Err(meta.error("duplicate sqlx_paginated attribute `crate`"));
                    }
                    crate_path = Some(value.parse()?);
                    Ok(())
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    Err(meta.error(format!(
                        "unknown sqlx_paginated container attribute `{path}`"
                    )))
                }
            })?;
        }

        Ok(Self {
            crate_path: crate_path.unwrap_or_else(|| syn::parse_quote!(::sqlx_paginated)),
        })
    }
}

enum FieldBehavior {
    Skip,
    Direct {
        variant_ident: Ident,
        output_name: String,
    },
    Flatten {
        variant_ident: Ident,
        ty: Type,
    },
}

fn parse_field(field: &Field, rename_all: Option<&str>) -> syn::Result<FieldBehavior> {
    let field_ident = field
        .ident
        .clone()
        .ok_or_else(|| syn::Error::new(field.span(), "expected named field"))?;

    let mut rename: Option<String> = None;
    let mut flatten = false;
    let mut skip = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("sqlx") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value: LitStr = meta.value()?.parse()?;
                if rename.is_some() {
                    return Err(meta.error("duplicate sqlx field attribute `rename`"));
                }
                rename = Some(value.value());
            } else if meta.path.is_ident("flatten") {
                if flatten {
                    return Err(meta.error("duplicate sqlx field attribute `flatten`"));
                }
                flatten = true;
            } else if meta.path.is_ident("skip") {
                if skip {
                    return Err(meta.error("duplicate sqlx field attribute `skip`"));
                }
                skip = true;
            } else {
                let path = meta.path.to_token_stream().to_string().replace(' ', "");
                return Err(meta.error(format!("unknown sqlx field attribute `{path}`")));
            }
            Ok(())
        })?;
    }

    if skip {
        return Ok(FieldBehavior::Skip);
    }

    let variant_ident = format_ident!("{}", field_ident.to_string().to_upper_camel_case());

    if flatten {
        return Ok(FieldBehavior::Flatten {
            variant_ident,
            ty: field.ty.clone(),
        });
    }

    let output_name = if let Some(name) = rename {
        name
    } else if let Some(rule) = rename_all {
        apply_rename_rule(&field_ident.to_string(), rule)?
    } else {
        field_ident.to_string()
    };

    Ok(FieldBehavior::Direct {
        variant_ident,
        output_name,
    })
}

fn parse_rename_all(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let mut rename_all = None;

    for attr in attrs {
        if !attr.path().is_ident("sqlx") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value: LitStr = meta.value()?.parse()?;
                if rename_all.is_some() {
                    return Err(meta.error("duplicate sqlx container attribute `rename_all`"));
                }
                rename_all = Some(value.value());
            }
            Ok(())
        })?;
    }

    Ok(rename_all)
}

fn apply_rename_rule(name: &str, rule: &str) -> syn::Result<String> {
    let value = match rule {
        "snake_case" => name.to_snake_case(),
        "camelCase" => name.to_lower_camel_case(),
        "PascalCase" => name.to_upper_camel_case(),
        "SCREAMING_SNAKE_CASE" => name.to_shouty_snake_case(),
        "kebab-case" => name.to_kebab_case(),
        "lowercase" => name.to_lowercase(),
        "UPPERCASE" => name.to_uppercase(),
        other => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("unsupported sqlx rename_all value: {other}"),
            ))
        }
    };
    Ok(value)
}

fn type_to_ident(ty: &Type) -> syn::Result<Ident> {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.clone())
            .ok_or_else(|| syn::Error::new(ty.span(), "unsupported flatten field type")),
        _ => Err(syn::Error::new(
            ty.span(),
            "flatten fields must be path types",
        )),
    }
}