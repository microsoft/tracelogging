use proc_macro::*;

use crate::expression::Expression;
use crate::field_option::FieldOption;

pub struct FieldInfo {
    pub type_name_span: Span,
    pub option: FieldOption,
    pub name: String,
    pub value_tokens: TokenStream,  // Context is type_name_span.
    pub intype_tokens: TokenStream, // Context is type_name_span.
    pub outtype: Expression,
    pub tag: Expression,
}
