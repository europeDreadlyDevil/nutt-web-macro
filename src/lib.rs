mod include_addr;

use core::panic;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use std::fmt::Debug;
use syn::__private::quote::quote;
use syn::__private::ToTokens;
use syn::{FnArg, ItemFn, Lit, Pat, PatType, Type, TypePath, TypeReference};

#[derive(Debug)]
enum ArgType {
    TypePath(TypePath),
    TypeRef(TypeReference),
}

impl ToTokens for ArgType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ArgType::TypePath(ty) => ty.to_tokens(tokens),
            ArgType::TypeRef(ty) => ty.to_tokens(tokens),
        }
    }
}

type FnItems = (
    ItemFn,
    Ident,
    String,
    Vec<Ident>,
    Vec<ArgType>,
    Vec<Ident>,
    Vec<ArgType>,
    Vec<Ident>,
    Vec<ArgType>,
    Vec<Ident>,
    Vec<ArgType>,
    Vec<Ident>,
    Vec<ArgType>,
);

fn get_fn_and_args_from_stream(attr: TokenStream, item: TokenStream) -> FnItems {
    let item = syn::parse::<ItemFn>(item.clone()).unwrap();
    let attr = syn::parse::<Lit>(attr).unwrap();
    let mut path = String::new();
    if let Lit::Str(lit) = attr {
        path = lit.value()
    } else {
        panic!("Path should be string")
    }
    let ident = item.clone().sig.ident;
    let args = item.clone().sig.inputs.into_iter().collect::<Vec<FnArg>>();
    let mut args_ident = vec![];
    let mut args_ty = vec![];
    let mut args_ident_state = vec![];
    let mut args_ident_json = vec![];
    let mut args_ty_json = vec![];
    let mut args_ty_state = vec![];
    let mut args_ident_session = vec![];
    let mut args_ty_session = vec![];
    let mut args_ident_cookie = vec![];
    let mut args_ty_cookie = vec![];

    for arg in &args {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let Pat::Ident(ident) = *pat.clone() {
                args_ident.push(ident.ident.clone());
            }
            if let Type::Path(ty) = *ty.clone() {
                let seg = ty.path.segments.iter().next().unwrap().clone().ident;
                if &seg.to_string() == "State" {
                    args_ty_state.push(ArgType::TypePath(ty.clone()));
                    if let Pat::Ident(ident) = *pat.clone() {
                        args_ident_state.push(ident.ident.clone());
                    }
                } else if &seg.to_string() == "CookieSession" {
                    args_ty_session.push(ArgType::TypePath(ty.clone()));
                    if let Pat::Ident(ident) = *pat.clone() {
                        args_ident_session.push(ident.ident.clone());
                    }
                } else if &seg.to_string() == "CookieJar" {
                    args_ty_cookie.push(ArgType::TypePath(ty.clone()));
                    if let Pat::Ident(ident) = *pat.clone() {
                        args_ident_cookie.push(ident.ident.clone());
                    }
                } else {
                    args_ty_json.push(ArgType::TypePath(ty.clone()));
                    if let Pat::Ident(ident) = *pat.clone() {
                        args_ident_json.push(ident.ident.clone());
                    }
                }
                args_ty.push(ArgType::TypePath(ty))
            }
        }
    }
    (
        item,
        ident,
        path,
        args_ident,
        args_ty,
        args_ident_json,
        args_ty_json,
        args_ident_state,
        args_ty_state,
        args_ident_session,
        args_ty_session,
        args_ident_cookie,
        args_ty_cookie,
    )
}

fn get_stream(method: &str, fn_items: FnItems) -> TokenStream {
    let (
        item,
        ident,
        path,
        args_ident,
        _args_ty,
        args_ident_json,
        args_ty_json,
        args_ident_state,
        args_ty_state,
        args_ident_session,
        args_ty_session,
        args_ident_cookie,
        args_ty_cookie,
    ) = fn_items;

    let vis = item.vis.clone();

    let method = match method {
        "get" => quote! { Method::GET },
        "post" => quote! {Method::POST},
        "put" => quote! {Method::PUT},
        "delete" => quote! {Method::DELETE},
        _ => panic!("Unhanding method"),
    };
    let stream = quote! {

        #vis fn #ident() -> Route {
            use std::future::Future;
            use std::pin::Pin;
            use nutt_web::http::method::Method;
            use nutt_web::http::request::Request;
            use nutt_web::modules::session::Session;
            use nutt_web::http::cookie::CookieJar;
            let f = |req: Request| -> Pin<Box<dyn Future<Output = Response> + Send + Sync>> {
                #item
                #(
                    let #args_ident_json: #args_ty_json = if let Ok(value) = req.body_json() {
                        value
                    } else { panic!("Args parsing error") };
                )*
                #(
                    let #args_ident_state: #args_ty_state = if let Some(value) = req.get_state().get(stringify!(#args_ident_state)) {
                        if let Some(value) = value.downcast_ref::<#args_ty_state>() {
                            value.clone()
                        } else {panic!("Downcast state type error")}
                    } else { panic!("Args parsing error") };
                )*
                #(
                    let #args_ident_session: #args_ty_session = {
                        if req.get_session().is_some() {
                            match req.get_session().as_ref() {
                                Some(Session::Cookie(session)) => {session.clone()}
                                None => {panic!("")}
                            }
                        } else { panic!("Args parsing error") }
                    };
                )*
                #(
                    let #args_ident_cookie: #args_ty_cookie = {
                        req.get_cookie_jar()
                    };
                )*
                Box::pin(#ident(#(#args_ident.clone(),)*))
            } as fn(Request) -> _;

            return Route::new(#method, #path, f)
        }
    };

    stream.into()
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_items = get_fn_and_args_from_stream(attr, item);
    get_stream("get", fn_items)
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_items = get_fn_and_args_from_stream(attr, item);
    get_stream("post", fn_items)
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_items = get_fn_and_args_from_stream(attr, item);
    get_stream("put", fn_items)
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_items = get_fn_and_args_from_stream(attr, item);
    get_stream("delete", fn_items)
}

#[proc_macro]
pub fn include_addr(_input: TokenStream) -> TokenStream {
    include_addr::include_addr(_input)
}
