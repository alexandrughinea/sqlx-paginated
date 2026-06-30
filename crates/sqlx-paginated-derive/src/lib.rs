use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Field, Fields, Ident,
    LitStr, Type,
};

#[proc_macro_derive(Fields, attributes(sqlx))]
pub fn derive_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = input.ident;
    let vis = input.vis;
    let enum_ident = format_ident!("{}Field", struct_ident);
    let rename_all = parse_rename_all(&input.attrs)?;

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
                    _ if #child_enum_ident::contains(value) => true,
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

        impl ::sqlx_paginated::FieldEnum for #enum_ident {
            fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms),*
                }
            }

            fn contains<S: AsRef<str>>(s: S) -> bool {
                match s.as_ref() {
                    #(#contains_arms)*
                    _ => false,
                }
            }
        }

        impl ::std::convert::Into<String> for #enum_ident {
            fn into(self) -> String {
                self.as_str().to_string()
            }
        }
    })
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
                rename = Some(value.value());
            } else if meta.path.is_ident("flatten") {
                flatten = true;
            } else if meta.path.is_ident("skip") {
                skip = true;
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
    for attr in attrs {
        if !attr.path().is_ident("sqlx") {
            continue;
        }

        let mut rename_all = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value: LitStr = meta.value()?.parse()?;
                rename_all = Some(value.value());
            }
            Ok(())
        })?;

        if rename_all.is_some() {
            return Ok(rename_all);
        }
    }

    Ok(None)
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

#[proc_macro_derive(Paginated)]
pub fn derive_paginated(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_ident = input.ident;
    let enum_ident = format_ident!("{}Field", struct_ident);
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::sqlx_paginated::Paginated for #struct_ident #ty_generics #where_clause {
            type Fields = #enum_ident;
        }
    };

    TokenStream::from(expanded)
}
