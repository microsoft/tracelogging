// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;

use crate::expression::Expression;
use crate::field_option::FieldOption;

pub struct FieldInfo {
    pub type_name_span: Span,
    pub option: &'static FieldOption,
    pub name: String,
    pub value_tokens: TokenStream,  // Context is type_name_span.
    pub intype_tokens: TokenStream, // Context is type_name_span. If empty use option.intype.
    pub outtype_or_field_count_expr: Expression, // If empty, use outtype_or_field_count_int
    pub outtype_or_field_count_int: u8, // Use only if outtype_or_field_count_expr is empty
    pub tag: Expression,
}
