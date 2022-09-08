// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;
use std::mem;
use std::vec;

use crate::strings::*;

pub struct Tree {
    trees: Vec<TokenTree>,
    span_stack: Vec<Span>,
    span: Span,
}

impl Tree {
    pub const fn new(span: Span) -> Self {
        return Self {
            trees: Vec::new(),
            span_stack: Vec::new(),
            span,
        };
    }

    pub fn push_span(&mut self, span: Span) -> &mut Self {
        self.span_stack.push(mem::replace(&mut self.span, span));
        return self;
    }

    pub fn pop_span(&mut self) -> &mut Self {
        self.span = self.span_stack.pop().unwrap();
        return self;
    }

    pub fn drain(&mut self) -> vec::Drain<TokenTree> {
        debug_assert!(self.span_stack.is_empty());
        return self.trees.drain(..);
    }

    pub fn add(&mut self, token: impl Into<TokenTree>) -> &mut Self {
        let mut tree = token.into();
        tree.set_span(self.span);
        self.trees.push(tree);
        return self;
    }

    pub fn add_punct(&mut self, chars: &str) -> &mut Self {
        let len = chars.len();
        for (index, ch) in chars.chars().enumerate() {
            let spacing = if index == len - 1 {
                Spacing::Alone
            } else {
                Spacing::Joint
            };
            self.add(Punct::new(ch, spacing));
        }
        return self;
    }

    pub fn add_ident(&mut self, name: &str) -> &mut Self {
        self.trees.push(Ident::new(name, self.span).into());
        return self;
    }

    pub fn add_path(&mut self, parts: &[&str]) -> &mut Self {
        for part in parts {
            self.add(Punct::new(':', Spacing::Joint));
            self.add(Punct::new(':', Spacing::Alone));
            self.add_ident(part);
        }
        return self;
    }

    pub fn add_outer_attribute(
        &mut self,
        name: &str,
        tokens: impl IntoIterator<Item = TokenTree>,
    ) -> &mut Self {
        let span = self.span;
        return self.add_punct("#").add_group_square([
            Ident::new(name, span).into(),
            Group::new(Delimiter::Parenthesis, TokenStream::from_iter(tokens)).into(),
        ]);
    }

    pub fn add_tokens(&mut self, tokens: impl IntoIterator<Item = TokenTree>) -> &mut Self {
        self.trees.extend(tokens);
        return self;
    }

    pub fn add_option_from_tokens(
        &mut self,
        tokens: impl IntoIterator<Item = TokenTree>,
    ) -> &mut Self {
        let stream = TokenStream::from_iter(tokens);
        if stream.is_empty() {
            self.add_path(OPTION_NONE_PATH);
        } else {
            self.add_path(OPTION_SOME_PATH)
                .add(Group::new(Delimiter::Parenthesis, stream));
        }
        return self;
    }

    pub fn add_group(
        &mut self,
        delimiter: Delimiter,
        tokens: impl IntoIterator<Item = TokenTree>,
    ) -> &mut Self {
        self.add(Group::new(delimiter, TokenStream::from_iter(tokens)));
        return self;
    }

    pub fn add_group_paren(&mut self, tokens: impl IntoIterator<Item = TokenTree>) -> &mut Self {
        return self.add_group(Delimiter::Parenthesis, tokens);
    }

    pub fn add_group_curly(&mut self, tokens: impl IntoIterator<Item = TokenTree>) -> &mut Self {
        return self.add_group(Delimiter::Brace, tokens);
    }

    pub fn add_group_square(&mut self, tokens: impl IntoIterator<Item = TokenTree>) -> &mut Self {
        return self.add_group(Delimiter::Bracket, tokens);
    }

    pub fn add_path_call(
        &mut self,
        path_parts: &[&str],
        group_tokens: impl IntoIterator<Item = TokenTree>,
    ) -> &mut Self {
        return self
            .add_path(path_parts)
            .add_group(Delimiter::Parenthesis, group_tokens);
    }

    pub fn add_sizeof(&mut self, type_name: &str) -> &mut Self {
        const MEM_SIZEOF: &[&str] = &["core", "mem", "size_of"];
        return self
            .add_path(MEM_SIZEOF)
            .add_punct("::")
            .add_punct("<")
            .add_ident(type_name)
            .add_punct(">")
            .add_group_paren(Vec::new().drain(..));
    }

    pub fn add_const_from_tokens(
        &mut self,
        var_name: &str,
        type_parts: &[&str],
        tokens: impl IntoIterator<Item = TokenTree>,
    ) -> &mut Self {
        return self
            .add_ident("const")
            .add_ident(var_name)
            .add_punct(":")
            .add_path(type_parts)
            .add_punct("=")
            .add_tokens(tokens)
            .add_punct(";");
    }

    /// If array_count == 0: `type_path`
    ///
    /// If array_count != 0: `[type_path; array_count]`
    pub fn add_scalar_type_path(
        &mut self,
        scratch_tree: &mut Tree,
        type_path: &[&str],
        array_count: u8,
    ) -> &mut Self {
        if array_count == 0 {
            // type_path
            self.add_path(type_path);
        } else {
            // [type_path; array_count]
            self.add_group_square(
                scratch_tree
                    .add_path(type_path)
                    .add_punct(";")
                    .add(Literal::usize_unsuffixed(array_count as usize))
                    .drain(),
            );
        }
        return self;
    }

    /// If array_count == 0: `identity::<&type_path>(value_tokens)`
    ///
    /// If array_count != 0: `identity::<&[type_path; array_count]>(value_tokens)`
    pub fn add_identity_call(
        &mut self,
        scratch_tree: &mut Tree,
        type_path: &[&str],
        array_count: u8,
        value_tokens: impl IntoIterator<Item = TokenTree>,
    ) -> &mut Self {
        return self
            .add_path(IDENTITY_PATH)
            .add_punct("::")
            .add_punct("<")
            .add_punct("&")
            .add_scalar_type_path(scratch_tree, type_path, array_count)
            .add_punct(">")
            .add_group_paren(value_tokens);
    }
}
