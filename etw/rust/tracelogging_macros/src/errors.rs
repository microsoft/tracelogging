// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;

pub struct Errors {
    error_tokens: Vec<TokenTree>,
}

impl Errors {
    pub const fn new() -> Self {
        return Self {
            error_tokens: Vec::new(),
        };
    }

    pub fn is_empty(&self) -> bool {
        return self.error_tokens.is_empty();
    }

    pub fn into_items(mut self) -> TokenStream {
        return self.error_tokens.drain(..).collect();
    }

    pub fn into_expression(mut self) -> TokenStream {
        self.error_tokens.push(Literal::u32_unsuffixed(0).into());
        return TokenTree::from(Group::new(
            Delimiter::Brace,
            self.error_tokens.drain(..).collect(),
        ))
        .into();
    }

    pub fn add(&mut self, pos: Span, error_message: &str) {
        self.add_token(pos, Punct::new(':', Spacing::Joint));
        self.add_token(pos, Punct::new(':', Spacing::Alone));
        self.add_token(pos, Ident::new("core", pos));
        self.add_token(pos, Punct::new(':', Spacing::Joint));
        self.add_token(pos, Punct::new(':', Spacing::Alone));
        self.add_token(pos, Ident::new("compile_error", pos));
        self.add_token(pos, Punct::new('!', Spacing::Alone));
        self.add_token(
            pos,
            Group::new(
                Delimiter::Parenthesis,
                TokenStream::from_iter([TokenTree::Literal(Literal::string(error_message))]),
            ),
        );
        self.add_token(pos, Punct::new(';', Spacing::Alone));
    }

    fn add_token(&mut self, pos: Span, token: impl Into<TokenTree>) {
        let mut tree = token.into();
        tree.set_span(pos);
        self.error_tokens.push(tree);
    }
}
