//! Internal utilities for handling procedural code generation for better extensibility.

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
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

                if let Some(attr) = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("setting"))
                {
                    let args: SettingAttr = attr.parse_args().unwrap();

                    if let Some(settings) = args.setting
                        && let Some(default) = settings.default.clone()
                    {
                        use_default = false;
                        defaults_list.extend(quote! {#identifier:#default,})
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

/// Generates a separate Godot class from the given struct, with exported properties based on the provided `setting` attributes.
/// This macro requires a struct name and Godot base class as input.
#[proc_macro_attribute]
pub fn settings_resource_from(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as SettingResourceAttr);
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_identifier = &input.ident;

    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let class_name = args.name;
            let base_class = args.base_class;

            let mut class_fields = quote! {};
            let mut setters = quote! {};
            let mut to_original_fields = quote! {};
            let mut from_original_fields = quote! {};

            for field in fields {
                let identifier = field.ident.as_ref().unwrap();

                // Fetch type
                let mut type_tokens = field.ty.to_token_stream();
                let mut type_conversion = quote! {};

                // Perform Rust -> Godot type conversions as necessary for the field type
                // Type conversions should work both ways
                (type_tokens, type_conversion) = match type_tokens.to_string().as_str() {
                    "Vec2" => (quote! {Vector2}, quote! {.to_vector2()}),
                    "Vec3" => (quote! {Vector3}, quote! {.to_vector3()}),
                    "Vec4" => (quote! {Vector4}, quote! {.to_vector4()}),
                    _ => (type_tokens, type_conversion),
                };

                let mut doc_comment = quote! {};
                if let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("doc")) {
                    doc_comment = attr.to_token_stream();
                }

                // Default field attributes
                let mut exporter = quote! {#[export]};
                let mut initializer = quote! {#[init(val=#type_tokens::default())]};

                if let Some(attr) = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("setting"))
                {
                    let args: SettingAttr = attr.parse_args().unwrap();

                    // Check if we have a default argument
                    if let Some(settings) = args.setting {
                        if let Some(min) = settings.min {
                            let mut range = quote! {#min};

                            if let Some(max) = settings.max {
                                range.extend(quote! {,#max});

                                if let Some(increment) = settings.incr {
                                    range.extend(quote! {,#increment});
                                }

                                if settings.soft_min {
                                    range.extend(quote! {,or_lesser});
                                }

                                if settings.soft_max {
                                    range.extend(quote! {,or_greater});
                                }

                                if let Some(unit) = settings.unit {
                                    range.extend(quote! {,suffix=#unit});
                                }

                                // Godot requires both minimum and maximum to be specified
                                exporter = quote! {#[export(range=(#range))]};
                            }
                        }

                        if let Some(default) = settings.default {
                            initializer = quote! {#[init(val=#default #type_conversion)]};
                        }
                    }
                }

                let setter_name_str = format!("set_{identifier}");
                let setter_name = syn::Ident::new(&setter_name_str, identifier.span());

                class_fields.extend(quote! {
                    #doc_comment
                    #[var(get, set = #setter_name)]
                    #exporter
                    #initializer
                    #identifier:#type_tokens,
                });

                setters.extend(quote! {
                    #[func]
                    fn #setter_name(&mut self, value: #type_tokens) {
                        self.#identifier = value;
                        self.base_mut().emit_changed();
                        self.signals().setting_changed().emit();
                    }
                });

                // Add a field for creating structs from this Resource
                // Type conversions should work both ways
                to_original_fields.extend(quote! {
                    #identifier: self.#identifier #type_conversion,
                });

                from_original_fields.extend(quote! {
                    self.#identifier = settings.#identifier #type_conversion;
                });
            }

            quote! {
                #input
                #[automatically_derived]
                #[cfg(feature = "godot")]
                #[derive(GodotClass)]
                #[class(init,base=#base_class,tool)]
                pub struct #class_name {
                    #class_fields
                    base: Base<#base_class>,
                }
                #[automatically_derived]
                #[cfg(feature = "godot")]
                #[godot_api]
                impl #class_name {
                    /// Emitted when any setting changes.
                    #[signal]
                    fn setting_changed();
                    #setters

                    /// Converts this resource into a corresponding pure Rust struct.
                    pub fn to_struct(&self) -> #struct_identifier {
                        #struct_identifier {
                            #to_original_fields
                        }
                    }

                    /// Applies the corresponding Pure rust struct to this Resource,
                    /// overriding all properties.
                    pub fn from_struct(&mut self, settings: #struct_identifier) {
                        #from_original_fields
                    }
                }
            }
        }
        _ => unimplemented!(),
    }
    .into()
}

struct SettingResourceAttr {
    name: Ident,
    base_class: Ident,
}

impl Parse for SettingResourceAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let base_class = input.parse()?;
        Ok(SettingResourceAttr { name, base_class })
    }
}
