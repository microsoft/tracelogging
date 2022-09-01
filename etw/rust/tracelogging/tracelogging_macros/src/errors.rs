use proc_macro::*;
use std::vec;

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

    pub fn drain(&mut self) -> vec::Drain<TokenTree> {
        return self.error_tokens.drain(..);
    }

    pub fn add(&mut self, pos: Span, error_message: &str) {
        if !self.error_tokens.is_empty() {
            self.add_token(pos, Punct::new(';', Spacing::Alone));
        }

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
    }

    fn add_token(&mut self, pos: Span, token: impl Into<TokenTree>) {
        let mut tree = token.into();
        tree.set_span(pos);
        self.error_tokens.push(tree);
    }
}
