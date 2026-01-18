use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, token::Comma, Expr, ExprLit, ExprTuple,
    ItemFn, Lit, Meta, Token,
};

struct HateoasArgs {
    args: Vec<(String, String)>,
    links: Vec<(String, String, String)>,
    query_fields: Vec<(String, String)>,
}

impl Parse for HateoasArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();
        let mut links = Vec::new();
        let mut query_fields = Vec::new();

        while !input.is_empty() {
            let meta: Meta = input.parse()?;

            match &meta {
                Meta::NameValue(nv) => {
                    let name = nv.path.get_ident().unwrap().to_string();
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                    {
                        args.push((name, s.value()));
                    }
                }
                Meta::List(list) => {
                    let name = list.path.get_ident().unwrap().to_string();
                    if name == "links" {
                        let content = list.tokens.clone();
                        let parser =
                            syn::punctuated::Punctuated::<ExprTuple, Comma>::parse_terminated;
                        if let Ok(tuples) = syn::parse::Parser::parse2(parser, content) {
                            for tuple in tuples {
                                if tuple.elems.len() == 3 {
                                    let vals: Vec<String> = tuple
                                        .elems
                                        .iter()
                                        .filter_map(|e| {
                                            if let Expr::Lit(ExprLit {
                                                lit: Lit::Str(s), ..
                                            }) = e
                                            {
                                                Some(s.value())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    if vals.len() == 3 {
                                        links.push((
                                            vals[0].clone(),
                                            vals[1].clone(),
                                            vals[2].clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    } else if name == "query_fields" {
                        let content = list.tokens.clone();
                        let parser =
                            syn::punctuated::Punctuated::<ExprTuple, Comma>::parse_terminated;
                        if let Ok(tuples) = syn::parse::Parser::parse2(parser, content) {
                            for tuple in tuples {
                                if tuple.elems.len() == 2 {
                                    let vals: Vec<String> = tuple
                                        .elems
                                        .iter()
                                        .filter_map(|e| {
                                            if let Expr::Lit(ExprLit {
                                                lit: Lit::Str(s), ..
                                            }) = e
                                            {
                                                Some(s.value())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    if vals.len() == 2 {
                                        query_fields.push((vals[0].clone(), vals[1].clone()));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(HateoasArgs {
            args,
            links,
            query_fields,
        })
    }
}

/// Macro for building simple HATEOAS responses
///
/// Usage:
/// ```
/// #[hateoas_simple(
///     resource = "events",
///     id_field = "id",
///     self_methods = "[GET, PUT, POST, DELETE]",
///     parent_methods = "[GET, POST]",
///     links(
///         ("event-packets", "event-packets", "[GET, POST]"),
///         ("tickets", "tickets", "[GET, POST]")
///     )
/// )]
/// pub fn build_simple_event(event: Event, base_url: &str) -> Response<Event> {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_simple(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let resource = args
        .args
        .iter()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v)
        .expect("resource parameter is required");
    let id_field = args
        .args
        .iter()
        .find(|(k, _)| k == "id_field")
        .map(|(_, v)| v)
        .expect("id_field parameter is required");
    let id_to_string = args
        .args
        .iter()
        .find(|(k, _)| k == "id_to_string")
        .map(|(_, v)| v);
    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v)
        .expect("self_methods parameter is required");
    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v)
        .expect("parent_methods parameter is required");

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let data_param = match fn_sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("First parameter must be the data object"),
    };

    let id_field_ident = syn::Ident::new(&id_field, proc_macro2::Span::call_site());

    let id_extraction = if let Some(conversion) = id_to_string {
        let conversion_tokens: proc_macro2::TokenStream =
            conversion.parse().expect("Invalid id_to_string");
        quote! {
            let id = #data_param.#id_field_ident.#conversion_tokens;
        }
    } else {
        quote! {
            let id = #data_param.#id_field_ident.clone();
        }
    };

    let link_calls = args.links.iter().map(|(name, path, methods)| {
        quote! {
            .link_with_types(
                #name,
                format!("{}/{}/{}/{}", base_url, #resource, id, #path),
                &[#methods]
            )
        }
    });

    let expanded = quote! {
        #fn_vis #fn_sig {
            #id_extraction
            ResponseBuilder::new(#data_param, format!("{}/{}/{}", base_url, #resource, id))
                .self_types(&[#self_methods])
                .parent_with_types(format!("{}/{}", base_url, #resource), &[#parent_methods])
                #(#link_calls)*
                .build()
        }
    };

    TokenStream::from(expanded)
}

/// Macro for building nested HATEOAS responses (resources under a parent)
///
/// Usage:
/// ```
/// #[hateoas_nested(
///     parent_resource = "events",
///     parent_id_field = "event_id",
///     resource = "tickets",
///     id_field = "cod",
///     self_methods = "[GET, PUT, POST, DELETE]",
///     parent_methods = "[GET, POST]"
/// )]
/// pub fn build_ticket_over_event(ticket: Ticket, event_id: i32, base_url: &str) -> Response<Ticket> {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_nested(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let parent_resource = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_resource")
        .map(|(_, v)| v)
        .expect("parent_resource parameter is required");
    let parent_id_field = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_id_field")
        .map(|(_, v)| v)
        .expect("parent_id_field parameter is required");
    let resource = args
        .args
        .iter()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v)
        .expect("resource parameter is required");
    let id_field = args
        .args
        .iter()
        .find(|(k, _)| k == "id_field")
        .map(|(_, v)| v)
        .expect("id_field parameter is required");
    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v)
        .expect("self_methods parameter is required");
    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v)
        .expect("parent_methods parameter is required");

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let data_param = match fn_sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("First parameter must be the data object"),
    };

    let id_field_ident = syn::Ident::new(&id_field, proc_macro2::Span::call_site());
    let parent_id_param = syn::Ident::new(&parent_id_field, proc_macro2::Span::call_site());

    let expanded = quote! {
        #fn_vis #fn_sig {
            let id = #data_param.#id_field_ident.clone();
            let self_url = format!("{}/{}/{}/{}/{}", base_url, #parent_resource, #parent_id_param, #resource, id);
            let parent_url = format!("{}/{}/{}/{}", base_url, #parent_resource, #parent_id_param, #resource);

            ResponseBuilder::new(#data_param, self_url)
                .self_types(&[#self_methods])
                .parent_with_types(parent_url, &[#parent_methods])
                .build()
        }
    };

    TokenStream::from(expanded)
}

/// Macro for building collection HATEOAS responses (resources listed under a parent)
///
/// Usage:
/// ```
/// #[hateoas_collection(
///     parent_resource = "events",
///     parent_id_field = "id",
///     resource = "event-packets",
///     self_methods = "[GET, POST]",
///     parent_methods = "[GET, PUT, POST, DELETE]"
/// )]
/// pub fn build_packet_over_event(packet: EventPackets, id: i32, base_url: &str) -> Response<EventPackets> {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_collection(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let parent_resource = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_resource")
        .map(|(_, v)| v)
        .expect("parent_resource parameter is required");
    let parent_id_field = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_id_field")
        .map(|(_, v)| v)
        .expect("parent_id_field parameter is required");
    let resource = args
        .args
        .iter()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v)
        .expect("resource parameter is required");
    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v)
        .expect("self_methods parameter is required");
    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v)
        .expect("parent_methods parameter is required");

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let data_param = match fn_sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("First parameter must be the data object"),
    };

    let parent_id_param = syn::Ident::new(&parent_id_field, proc_macro2::Span::call_site());

    let expanded = quote! {
        #fn_vis #fn_sig {
            let self_url = format!("{}/{}/{}/{}", base_url, #parent_resource, #parent_id_param, #resource);

            ResponseBuilder::new(#data_param, self_url)
                .self_types(&[#self_methods])
                .parent_with_types(
                    format!("{}/{}/{}", base_url, #parent_resource, #parent_id_param),
                    &[#parent_methods]
                )
                .build()
        }
    };

    TokenStream::from(expanded)
}

/// Macro for building filtered HATEOAS responses with query parameters
///
/// Usage:
/// ```
/// #[hateoas_filtered(
///     resource = "events",
///     self_methods = "GET",
///     parent_methods = "[GET, POST]",
///     query_fields(
///         ("locatie", "location"),
///         ("nume", "name")
///     )
/// )]
/// pub fn build_filtered_event(events: Vec<Event>, params: &EventQuery, base_url: &str) -> Vec<Response<Event>> {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_filtered(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let resource = args
        .args
        .iter()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v)
        .expect("resource parameter is required");
    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v)
        .expect("self_methods parameter is required");
    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v)
        .expect("parent_methods parameter is required");

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let data_param = match fn_sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("First parameter must be the data collection"),
    };

    let query_param = match fn_sig.inputs.iter().nth(1) {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("Second parameter must be the query params"),
    };

    let query_checks: Vec<_> = args
        .query_fields
        .iter()
        .map(|(field, query_name)| {
            let field_parts: Vec<&str> = field.split('.').collect();
            let field_access = if field_parts.len() == 2 {
                let first = syn::Ident::new(field_parts[0], proc_macro2::Span::call_site());
                let second = syn::Ident::new(field_parts[1], proc_macro2::Span::call_site());
                quote! { #query_param.#first.#second }
            } else {
                let field_ident = syn::Ident::new(field, proc_macro2::Span::call_site());
                quote! { #query_param.#field_ident }
            };

            quote! {
                if let Some(val) = &#field_access {
                    query_parts.push(format!(concat!(#query_name, "={}"), val));
                }
            }
        })
        .collect();

    let query_checks_clone = query_checks.clone();

    let has_pagination = args
        .query_fields
        .iter()
        .any(|(field, _)| field == "paginare.page");

    let expanded = if has_pagination {
        quote! {
            #fn_vis #fn_sig {
                let mut responses = Vec::new();

                let has_page_field = #query_param.paginare.page.is_some();
                let current_page = #query_param.paginare.page.unwrap_or(1);
                let items_per_page = #query_param.paginare.items_per_page.unwrap_or(10);
                let item_count = #data_param.len();

                // Repository fetches items_per_page + 1 to check if there's a next page
                // If we got more than items_per_page, there's a next page
                let has_next_page = item_count > items_per_page as usize;

                // Only iterate over items_per_page items (skip the extra one used for next page detection)
                for item in #data_param.into_iter().take(items_per_page as usize) {
                    let mut self_href = format!("{}/{}", base_url, #resource);
                    let mut query_parts = vec![];

                    #(#query_checks)*

                    if !query_parts.is_empty() {
                        self_href = format!("{}?{}", self_href, query_parts.join("&"));
                    }

                    let mut response_builder = ResponseBuilder::new(item, self_href)
                        .self_types(&[#self_methods])
                        .parent_with_types(format!("{}/{}", base_url, #resource), &[#parent_methods]);

                    if has_page_field {
                        let mut non_page_params = vec![];
                        #(#query_checks_clone)*
                        non_page_params.retain(|p: &String| !p.starts_with("page=") && !p.starts_with("items_per_page="));

                        let base_with_params = if non_page_params.is_empty() {
                            format!("{}/{}", base_url, #resource)
                        } else {
                            format!("{}/{}?{}", base_url, #resource, non_page_params.join("&"))
                        };

                        if current_page > 1 {
                            let prev_page = current_page - 1;
                            let prev_url = if non_page_params.is_empty() {
                                format!("{}?page={}&items_per_page={}", base_with_params, prev_page, items_per_page)
                            } else {
                                format!("{}&page={}&items_per_page={}", base_with_params, prev_page, items_per_page)
                            };
                            response_builder = response_builder.link_with_types("prev", prev_url, &["GET"]);
                        }

                        // Only add next link if there's actually a next page
                        if has_next_page {
                            let next_page = current_page + 1;
                            let next_url = if non_page_params.is_empty() {
                                format!("{}?page={}&items_per_page={}", base_with_params, next_page, items_per_page)
                            } else {
                                format!("{}&page={}&items_per_page={}", base_with_params, next_page, items_per_page)
                            };
                            response_builder = response_builder.link_with_types("next", next_url, &["GET"]);
                        }
                    }

                    responses.push(response_builder.build());
                }

                responses
            }
        }
    } else {
        // without pagination
        quote! {
            #fn_vis #fn_sig {
                let mut responses = Vec::new();

                for item in #data_param {
                    let mut self_href = format!("{}/{}", base_url, #resource);
                    let mut query_parts = vec![];

                    #(#query_checks)*

                    if !query_parts.is_empty() {
                        self_href = format!("{}?{}", self_href, query_parts.join("&"));
                    }

                    let response = ResponseBuilder::new(item, self_href)
                        .self_types(&[#self_methods])
                        .parent_with_types(format!("{}/{}", base_url, #resource), &[#parent_methods])
                        .build();

                    responses.push(response);
                }

                responses
            }
        }
    };

    TokenStream::from(expanded)
}

/// Macro for building Links structures directly (for legacy compatibility)
///
/// This macro is used for helper functions that return raw Links structures instead of Response<T>.
/// It supports flexible path parameters, optional links, and HTTP method annotations.
///
/// Usage:
/// ```
/// #[hateoas_links(
///     self_path = "{}/api/client-manager/clients/{}",
///     self_methods = "[GET, PUT, PATCH, DELETE]",
///     parent_path = "{}/api/client-manager/clients",
///     parent_methods = "[GET, POST]",
///     links(
///         ("tickets", "{}/api/client-manager/clients/{}/tickets", "[GET, POST]")
///     )
/// )]
/// pub fn client_links(base_url: &str, client_id: &str) -> Links {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_links(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let self_path = args
        .args
        .iter()
        .find(|(k, _)| k == "self_path")
        .map(|(_, v)| v)
        .expect("self_path parameter is required");

    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v);

    let parent_path = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_path")
        .map(|(_, v)| v);

    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v);

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let param_names: Vec<_> = fn_sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(pat_ident.ident.clone());
                }
            }
            None
        })
        .collect();

    let count_placeholders = |s: &str| s.matches("{}").count();

    let parent_link = if let Some(pp) = parent_path {
        let parent_count = count_placeholders(pp);
        let parent_params: Vec<_> = param_names.iter().take(parent_count).collect();

        if let Some(pm) = parent_methods {
            quote! { Some(Link::new(format!(#pp, #(#parent_params),*)).with_types(&[#pm])) }
        } else {
            quote! { Some(Link::new(format!(#pp, #(#parent_params),*))) }
        }
    } else {
        quote! { None }
    };

    let self_count = count_placeholders(self_path);
    let self_params: Vec<_> = param_names.iter().take(self_count).collect();
    let self_link = if let Some(sm) = self_methods {
        quote! { Link::new(format!(#self_path, #(#self_params),*)).with_types(&[#sm]) }
    } else {
        quote! { Link::new(format!(#self_path, #(#self_params),*)) }
    };

    let others_map = if args.links.is_empty() {
        quote! { None }
    } else {
        let link_entries = args.links.iter().map(|(name, path, methods)| {
            let link_count = count_placeholders(path);
            let link_params: Vec<_> = param_names.iter().take(link_count).collect();

            if !methods.is_empty() {
                quote! {
                    (#name.to_string(), Link::new(format!(#path, #(#link_params),*)).with_types(&[#methods]))
                }
            } else {
                quote! {
                    (#name.to_string(), Link::new(format!(#path, #(#link_params),*)))
                }
            }
        });
        quote! {
            Some(std::collections::HashMap::from([
                #(#link_entries),*
            ]))
        }
    };

    let expanded = quote! {
        #fn_vis #fn_sig {
            Links {
                parent: #parent_link,
                link: #self_link,
                others: #others_map,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Macro for building HATEOAS responses for action endpoints (login, register, etc.)
///
/// This macro is designed for endpoints where the self URL is a fixed action path,
/// not based on an ID field in the response data. It supports optional parent links
/// and additional related links.
///
/// Usage:
/// ```
/// #[hateoas_action(
///     resource = "api/auth/login",
///     self_methods = "POST",
///     links(
///         ("register", "api/auth/register", "POST"),
///         ("my-client", "api/client-manager/clients/me", "GET")
///     )
/// )]
/// pub fn build_login_response(response: LoginResponse, base_url: &str) -> Response<LoginResponse> {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let resource = args
        .args
        .iter()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v)
        .expect("resource parameter is required");

    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v)
        .expect("self_methods parameter is required");

    let parent_resource = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_resource")
        .map(|(_, v)| v);

    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v);

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let data_param = match fn_sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("First parameter must be the data object"),
    };

    let link_calls = args.links.iter().map(|(name, path, methods)| {
        quote! {
            .link_with_types(
                #name,
                format!("{}/{}", base_url, #path),
                &[#methods]
            )
        }
    });

    let parent_call = if let (Some(pr), Some(pm)) = (parent_resource, parent_methods) {
        quote! {
            .parent_with_types(format!("{}/{}", base_url, #pr), &[#pm])
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #fn_vis #fn_sig {
            ResponseBuilder::new(#data_param, format!("{}/{}", base_url, #resource))
                .self_types(&[#self_methods])
                #parent_call
                #(#link_calls)*
                .build()
        }
    };

    TokenStream::from(expanded)
}

/// Macro for building HATEOAS responses for data lookup endpoints
///
/// This macro is for endpoints where the self URL includes a lookup parameter
/// passed as a separate function argument (not from the data object).
///
/// Usage:
/// ```
/// #[hateoas_lookup(
///     resource = "api/client-manager/clients/data",
///     lookup_param = "ticket_code",
///     self_methods = "GET",
///     parent_resource = "api/client-manager/clients",
///     parent_methods = "[GET, POST]"
/// )]
/// pub fn build_ticket_buyer_info(buyer_info: TicketBuyerInfo, ticket_code: &str, base_url: &str) -> Response<TicketBuyerInfo> {}
/// ```
#[proc_macro_attribute]
pub fn hateoas_lookup(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as HateoasArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let resource = args
        .args
        .iter()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v)
        .expect("resource parameter is required");

    let lookup_param = args
        .args
        .iter()
        .find(|(k, _)| k == "lookup_param")
        .map(|(_, v)| v)
        .expect("lookup_param parameter is required");

    let self_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "self_methods")
        .map(|(_, v)| v)
        .expect("self_methods parameter is required");

    let parent_resource = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_resource")
        .map(|(_, v)| v);

    let parent_methods = args
        .args
        .iter()
        .find(|(k, _)| k == "parent_methods")
        .map(|(_, v)| v);

    let fn_sig = &input_fn.sig;
    let fn_vis = &input_fn.vis;

    let data_param = match fn_sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => &pat_type.pat,
        _ => panic!("First parameter must be the data object"),
    };

    let lookup_param_ident = syn::Ident::new(&lookup_param, proc_macro2::Span::call_site());

    let link_calls = args.links.iter().map(|(name, path, methods)| {
        quote! {
            .link_with_types(
                #name,
                format!("{}/{}", base_url, #path),
                &[#methods]
            )
        }
    });

    let parent_call = if let (Some(pr), Some(pm)) = (parent_resource, parent_methods) {
        quote! {
            .parent_with_types(format!("{}/{}", base_url, #pr), &[#pm])
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #fn_vis #fn_sig {
            ResponseBuilder::new(#data_param, format!("{}/{}/{}", base_url, #resource, #lookup_param_ident))
                .self_types(&[#self_methods])
                #parent_call
                #(#link_calls)*
                .build()
        }
    };

    TokenStream::from(expanded)
}
