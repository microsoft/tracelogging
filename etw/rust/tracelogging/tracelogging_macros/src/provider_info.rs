// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;

use crate::errors::Errors;
use crate::guid::Guid;
use crate::parser::{ArgConstraints::*, ArgResult, Parser};

pub struct ProviderInfo {
    pub symbol: Ident,
    pub name: String,
    pub id: Guid,
    pub group_id: Option<Guid>,
    pub debug: bool,
}

impl ProviderInfo {
    pub fn try_from_tokens(
        arg_span: Span,
        arg_tokens: TokenStream,
    ) -> Result<ProviderInfo, TokenStream> {
        let mut prov_id_set = false;
        let mut errors = Errors::new();
        let mut root_parser = Parser::new(&mut errors, arg_span, arg_tokens);
        let mut prov = ProviderInfo {
            name: String::new(),
            id: Guid::zero(),
            group_id: None,
            debug: false,
            symbol: Ident::new("x", arg_span),
        };

        // symbol name

        if let Some(ident) = root_parser.next_ident(
            RequiredNotLast,
            "expected identifier for provider symbol, e.g. MY_PROVIDER",
        ) {
            prov.symbol = ident;
        }

        // provider name

        if let Some((prov_name, span)) = root_parser.next_string_literal(
            Required,
            "expected string literal for provider name, e.g. define_provider!(MY_PROVIDER, \"MyCompany.MyComponent\")",
        ) {
            prov.name = prov_name;
            if prov.name.len() >= 32768 {
                root_parser.errors().add(span, "provider name.len() must be less than 32KB");
            }
            if prov.name.contains('\0') {
                root_parser.errors().add(span, "provider name must not contain '\\0'");
            }
        }

        // provider options (id or group_id)

        while let ArgResult::Option(option_name_ident, mut option_args_parser) =
            root_parser.next_arg(false)
        {
            let errors = option_args_parser.errors();
            let id_dest = match option_name_ident.to_string().as_str() {
                "debug" => {
                    prov.debug = true;
                    continue;
                }
                "id" => {
                    if prov_id_set {
                        errors.add(option_name_ident.span(), "id already set");
                    }
                    prov_id_set = true;
                    &mut prov.id
                }
                "group_id" | "groupid" => {
                    if prov.group_id.is_some() {
                        errors.add(option_name_ident.span(), "group_id already set");
                    }
                    prov.group_id.insert(Guid::zero())
                }
                _ => {
                    errors.add(
                        option_name_ident.span(),
                        "expected id(\"GUID\") or group_id(\"GUID\")",
                    );
                    continue;
                }
            };

            const EXPECTED_GUID: &str =
                "expected \"GUID\", e.g. \"20cf46dd-3b90-476c-94e9-4e74bbc30e31\"";
            if let Some((id_str, id_span)) =
                option_args_parser.next_string_literal(RequiredLast, EXPECTED_GUID)
            {
                if let Some(id_val) = Guid::try_parse(&id_str) {
                    *id_dest = id_val;
                } else {
                    option_args_parser.errors().add(id_span, EXPECTED_GUID);
                }
            }
        }

        if !prov_id_set {
            prov.id = Guid::from_name(&prov.name);
        }

        return if errors.is_empty() {
            Ok(prov)
        } else {
            Err(errors.drain().collect())
        };
    }
}
