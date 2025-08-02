//! Internal utilities for handling procedural code generation for better extensibility.

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Error, Expr, Ident, LitFloat, LitStr, Token};
// https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros
// https://www.freecodecamp.org/news/procedural-macros-in-rust/#heading-the-intostringhashmap-derive-macro

/// Settings management with sensible defaults.
#[proc_macro_derive(ExposeSettings, attributes(setting))]
pub fn expose_settings_fn(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_identifier = &input.ident;

    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let mut defaults_list = quote! {};

            for field in fields {
                let identifier = field.ident.as_ref().unwrap();
                let mut use_default: bool = true;

                // hash_map.insert(stringify!(#identifier).to_string(), String::from(value.#identifier));
                if let Some(attr) = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("setting"))
                {
                    let args: SettingAttr = attr.parse_args().unwrap();

                    if let Some(settings) = args.setting {
                        if let Some(default) = settings.default {
                            use_default = false;
                            defaults_list.extend(quote! {#identifier:#default,})
                        }
                    }
                }

                if use_default {
                    let type_tokens = field.ty.to_token_stream();
                    defaults_list.extend(quote! {#identifier:#type_tokens::default(),});
                }
            }

            quote! {
                #[automatically_derived]
                impl Default for #struct_identifier {
                    fn default() -> Self {
                        #struct_identifier {
                            #defaults_list
                        }
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

struct Setting {
    default: Option<Expr>,
    min: Option<LitFloat>,
    max: Option<LitFloat>,
    incr: Option<LitFloat>,
    soft_min: bool,
    soft_max: bool,
    unit: Option<String>,
}

struct SettingAttr {
    setting: Option<Setting>,
}

impl Parse for SettingAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut default: Option<Expr> = None;
        let mut min: Option<LitFloat> = None;
        let mut max: Option<LitFloat> = None;
        let mut incr: Option<LitFloat> = None;
        let mut soft_min = false;
        let mut soft_max = false;
        let mut unit: Option<String> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match &*ident.to_string() {
                "default" => {
                    input.parse::<Token![=]>()?;
                    default = Some(input.parse()?);
                }
                "min" => {
                    input.parse::<Token![=]>()?;
                    if let Ok(lit) = input.parse::<LitFloat>() {
                        min = Some(lit);
                    } else {
                        return Err(Error::new(
                            ident.span(),
                            "Expected a float literal for 'min'",
                        ));
                    }
                }
                "max" => {
                    input.parse::<Token![=]>()?;
                    if let Ok(lit) = input.parse::<LitFloat>() {
                        max = Some(lit);
                    } else {
                        return Err(Error::new(
                            ident.span(),
                            "Expected a float literal for 'max'",
                        ));
                    }
                }
                "incr" => {
                    input.parse::<Token![=]>()?;
                    if let Ok(lit) = input.parse::<LitFloat>() {
                        incr = Some(lit);
                    } else {
                        return Err(Error::new(
                            ident.span(),
                            "Expected a float literal for 'incr'",
                        ));
                    }
                }
                "soft_min" => soft_min = true,
                "soft_max" => soft_max = true,
                "unit" => {
                    input.parse::<Token![=]>()?;
                    if let Ok(lit) = input.parse::<LitStr>() {
                        unit = Some(lit.value());
                    } else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "Expected a string literal for 'unit'",
                        ));
                    }
                }
                _ => return Err(syn::Error::new_spanned(ident, "Unknown attribute")),
            }

            // Check if there's a comma and consume it
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(SettingAttr {
            setting: Some(Setting {
                default,
                min,
                max,
                incr,
                soft_min,
                soft_max,
                unit,
            }),
        })
    }
}
