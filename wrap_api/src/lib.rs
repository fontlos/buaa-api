use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, Attribute, DeriveInput, Ident, Token,
};

struct WrapApiArgs {
    constructor_fn: Ident,
    with_vpn: bool,
}

impl Parse for WrapApiArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let constructor_fn: Ident = input.parse()?;
        let with_vpn = if input.peek(Token![,]) && input.peek2(Ident) {
            input.parse::<Token![,]>()?;
            input.parse::<Ident>()? == "vpn"
        } else {
            false
        };
        Ok(WrapApiArgs {
            constructor_fn,
            with_vpn,
        })
    }
}

#[proc_macro_attribute]
pub fn wrap_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as WrapApiArgs);
    let item_input = parse_macro_input!(item as DeriveInput);

    let struct_name = &item_input.ident;
    let constructor_fn = args.constructor_fn;
    let with_vpn = args.with_vpn;

    // 提取文档注释
    let original_attrs: Vec<Attribute> = item_input.attrs;

    // 生成 VPN 相关的代码
    let vpn_struct_name = if with_vpn {
        Some(Ident::new(
            &format!("{}VPN", struct_name),
            struct_name.span(),
        ))
    } else {
        None
    };

    let vpn_code = if let Some(vpn_struct_name) = vpn_struct_name {
        quote! {
            impl #struct_name {
                pub fn vpn(&self) -> #vpn_struct_name {
                    #vpn_struct_name {
                        client: self.client.clone(),
                        cookies: self.cookies.clone(),
                        config: self.config.clone(),
                    }
                }
            }

            pub struct #vpn_struct_name {
                client: reqwest::Client,
                cookies: std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>,
                pub config: std::sync::Arc<std::sync::RwLock<crate::Config>>,
            }

            impl std::ops::Deref for #vpn_struct_name {
                type Target = reqwest::Client;

                fn deref(&self) -> &Self::Target {
                    &self.client
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #( #original_attrs )* // 文档注释
        pub struct #struct_name {
            client: reqwest::Client,
            cookies: std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>,
            pub config: std::sync::Arc<std::sync::RwLock<crate::Config>>,
        }

        impl std::ops::Deref for #struct_name {
            type Target = reqwest::Client;

            fn deref(&self) -> &Self::Target {
                &self.client
            }
        }

        impl crate::Context {
            #( #original_attrs )*
            pub fn #constructor_fn(&self) -> #struct_name {
                #struct_name {
                    client: self.client.clone(),
                    cookies: self.cookies.clone(),
                    config: self.config.clone(),
                }
            }
        }

        #vpn_code
    };

    TokenStream::from(expanded)
}
