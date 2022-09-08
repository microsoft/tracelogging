// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;

pub struct Expression {
    pub context: Span,
    pub tokens: TokenStream,
}

impl Expression {
    pub fn empty(context: Span) -> Self {
        return Self {
            context,
            tokens: TokenStream::new(),
        };
    }

    pub const fn new(context: Span, tokens: TokenStream) -> Self {
        return Self { context, tokens };
    }

    pub fn is_empty(&self) -> bool {
        return self.tokens.is_empty();
    }
}
