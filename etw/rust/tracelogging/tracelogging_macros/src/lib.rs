// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![allow(clippy::needless_return)]

//! Implements the macros that are exported by the tracelogging crate.

extern crate proc_macro;
use proc_macro::{Literal, Span, TokenStream, TokenTree};

use crate::event_generator::EventGenerator;
use crate::event_info::EventInfo;
use crate::provider_generator::ProviderGenerator;
use crate::provider_info::ProviderInfo;

#[proc_macro]
pub fn define_provider(arg_tokens: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    return match ProviderInfo::try_from_tokens(call_site, arg_tokens) {
        Err(error_tokens) => error_tokens,
        Ok(prov) => ProviderGenerator::new(call_site).generate(prov),
    };
}

#[proc_macro]
pub fn write_event(arg_tokens: TokenStream) -> TokenStream {
    let call_site = Span::call_site();
    return match EventInfo::try_from_tokens(call_site, arg_tokens) {
        Err(error_tokens) => error_tokens,
        Ok(prov) => EventGenerator::new(call_site).generate(prov),
    };
}

/// For testing: `define_provider2!(ignored)` --> nothing
#[proc_macro]
pub fn define_provider2(_arg_tokens: TokenStream) -> TokenStream {
    return TokenStream::new();
}

/// For testing: `write_event2!(ignored)` --> `0`
#[proc_macro]
pub fn write_event2(_arg_tokens: TokenStream) -> TokenStream {
    return TokenTree::Literal(Literal::u32_unsuffixed(0)).into();
}

// The tracelogging crate depends on the tracelogging_macros crate so the
// tracelogging_macros crate can't depend on the tracelogging crate. Instead, pull in
// the source code for needed modules.

#[allow(dead_code)]
#[path = "../../src/guid.rs"]
mod guid;

mod enums;
mod errors;
mod event_generator;
mod event_info;
mod expression;
mod field_info;
mod field_option;
mod field_options;
mod ident_builder;
mod parser;
mod provider_generator;
mod provider_info;
mod strings;
mod tree;
