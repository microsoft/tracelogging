// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;
use std::iter;
use std::str;

use crate::errors::Errors;

use ArgConstraints::*;

pub struct Parser<'a> {
    iterator: token_stream::IntoIter,
    most_recent_span: Span,
    errors: &'a mut Errors,
}

impl<'a> Parser<'a> {
    pub fn new(errors: &'a mut Errors, context: Span, tokens: TokenStream) -> Self {
        return Self {
            iterator: tokens.into_iter(),
            most_recent_span: context,
            errors,
        };
    }

    fn from_group(errors: &'a mut Errors, group: Group) -> Self {
        let group_span = group.span();
        return Self {
            iterator: group.stream().into_iter(),
            most_recent_span: group_span,
            errors,
        };
    }

    pub fn errors(&mut self) -> &mut Errors {
        return self.errors;
    }

    pub fn move_next(&mut self) -> Option<TokenTree> {
        let current = self.iterator.next();
        if let Some(token) = &current {
            self.most_recent_span = token.span();
        }
        return current;
    }

    /// Moves to the next comma or the end-of-stream.
    /// Emits "expected ..." error for other tokens encountered before comma or end-of-stream.
    pub fn next_comma(&mut self, constraints: ArgConstraints) {
        match self.move_next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                self.comma_after_item(constraints)
            }
            Some(token) => {
                self.unexpected_token_before_end(constraints, token.span());
                self.skip_to_comma(token);
            }
            None => (),
        };
    }

    /// Reads an identifier then moves to the next comma or the end-of-stream.
    /// Emits "expected ..." error for other tokens encountered before comma or end-of-stream.
    pub fn next_ident(
        &mut self,
        constraints: ArgConstraints,
        error_message: &str,
    ) -> Option<Ident> {
        let result;
        match self.move_next() {
            Some(TokenTree::Ident(ident)) => {
                result = Some(ident);
                self.next_comma(constraints);
            }
            Some(token) => {
                self.errors.add(token.span(), error_message);
                if self.skip_to_comma(token) {
                    self.comma_after_item(constraints);
                }
                result = None;
            }
            None => {
                self.eos_before_item(constraints, error_message);
                result = None;
            }
        }
        return result;
    }

    /// Reads a string literal then moves to the next comma or the end-of-stream.
    /// Emits "expected ..." error for other tokens encountered before comma or end-of-stream.
    pub fn next_string_literal(
        &mut self,
        constraints: ArgConstraints,
        error_message: &str,
    ) -> Option<(String, Span)> {
        let result;
        match self.move_next() {
            Some(TokenTree::Literal(literal)) => {
                let lit_str = literal.to_string();
                if lit_str.len() < 2 || !lit_str.starts_with('"') || !lit_str.ends_with('"') {
                    self.errors.add(literal.span(), error_message);
                    if self.skip_to_comma(TokenTree::Literal(literal)) {
                        self.comma_after_item(constraints);
                    }
                    result = None;
                } else {
                    if let Some(unescaped) = unescape(&lit_str[1..lit_str.len() - 1]) {
                        result = Some((unescaped, literal.span()));
                    } else {
                        self.errors
                            .add(literal.span(), "unsupported escape sequence");
                        result = None;
                    }
                    self.next_comma(constraints);
                }
            }
            Some(token) => {
                self.errors.add(token.span(), error_message);
                if self.skip_to_comma(token) {
                    self.comma_after_item(constraints);
                }
                result = None;
            }
            None => {
                self.eos_before_item(constraints, error_message);
                result = None;
            }
        }
        return result;
    }

    /// Reads tokens to the next comma or the end-of-stream.
    /// Emits an error if no tokens or if ';'.
    pub fn next_tokens(&mut self, constraints: ArgConstraints, error_message: &str) -> TokenStream {
        enum State {
            GotNone,
            GotSome,
            Done,
        }

        let mut state = State::GotNone;
        return TokenStream::from_iter(iter::from_fn(|| {
            if let State::Done = state {
                return None;
            } else {
                match self.move_next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                        // Treat ',' as end-of-arg.
                        if let State::GotNone = state {
                            self.errors.add(punct.span(), error_message);
                        }
                        self.comma_after_item(constraints);
                        state = State::Done;
                        return None;
                    }
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ';' => {
                        // Treat ';' as invalid at top level of arguments.
                        self.unexpected_token_before_end(constraints, punct.span());
                        self.skip_to_comma(punct.into());
                        state = State::Done;
                        return None;
                    }
                    Some(token) => {
                        state = State::GotSome;
                        return Some(token);
                    }
                    None => {
                        if let State::GotNone = state {
                            self.eos_before_item(constraints, error_message);
                        }
                        state = State::Done;
                        return None;
                    }
                }
            }
        }));
    }

    /// Reads OptionIdent(ArgsGroup) or {...} then moves to the next comma or the end-of-stream.
    /// Emits "expected option" errors for non-option syntax.
    /// Emits "expected ..." error for other tokens encountered before comma or end-of-stream.
    pub fn next_arg(&mut self, want_struct: bool) -> ArgResult {
        const EXPECTED_OPTION: &str = "expected identifier for option name, e.g. Option(args...)";
        const EXPECTED_OPTION_OR_STRUCT: &str =
            "expected '{' for struct or identifier for option name, e.g. Option(args...)";
        const EXPECTED_OPTION_ARGS: &str = "expected '(' after option name, e.g. Option(args...)";

        let result;
        loop {
            // Expect: OptionName or {}

            let first_token = self.move_next();
            match first_token {
                Some(TokenTree::Group(struct_group))
                    if want_struct && struct_group.delimiter() == Delimiter::Brace =>
                {
                    result = ArgResult::Struct(Parser::from_group(self.errors, struct_group));
                    break;
                }
                Some(TokenTree::Ident(name_ident)) => {
                    // Expect: (option_args)

                    let args_token = self.move_next();
                    let args_group = match args_token {
                        Some(TokenTree::Group(group))
                            if group.delimiter() == Delimiter::Parenthesis =>
                        {
                            group
                        }
                        Some(token) => {
                            self.errors.add(token.span(), EXPECTED_OPTION_ARGS);
                            self.skip_to_comma(token);
                            continue;
                        }
                        None => {
                            self.errors.add(name_ident.span(), EXPECTED_OPTION_ARGS);
                            result = ArgResult::None;
                            break;
                        }
                    };

                    self.next_comma(Optional); // Assume options are optional.
                    result =
                        ArgResult::Option(name_ident, Parser::from_group(self.errors, args_group));
                    break;
                }
                Some(token) => {
                    self.errors.add(
                        token.span(),
                        if want_struct {
                            EXPECTED_OPTION_OR_STRUCT
                        } else {
                            EXPECTED_OPTION
                        },
                    );
                    self.skip_to_comma(token);
                    continue;
                }
                None => {
                    if !want_struct {
                        // Assume options are optional.
                    } else {
                        self.errors
                            .add(self.most_recent_span, EXPECTED_OPTION_OR_STRUCT);
                    }

                    result = ArgResult::None;
                    break;
                }
            };
        }

        return result;
    }

    /// Typically called to recover from a syntax error.
    /// Returns true for comma, false for end-of-stream.
    /// If skip_to_comma returns true, you may want to call comma_after_item.
    ///
    /// `while current() != ',' && current() != None { move_next(); }`
    fn skip_to_comma(&mut self, next_token: TokenTree) -> bool {
        let result;
        let mut current = Some(next_token);
        loop {
            match current {
                None => {
                    result = false;
                    break;
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                    result = true;
                    break;
                }
                _ => current = self.move_next(),
            };
        }
        return result;
    }

    /// If the item was required then emit an error message.
    fn eos_before_item(&mut self, constraints: ArgConstraints, error_message: &str) {
        match constraints {
            Optional /*| OptionalLast*/ => (),
            _ => self.errors.add(self.most_recent_span, error_message),
        }
    }

    /// If item must be last and isn't then emit `expected ')'`.
    fn comma_after_item(&mut self, constraints: ArgConstraints) {
        if let RequiredLast /*| OptionalLast*/ = constraints {
            if self.move_next().is_some() {
                self.unexpected_token_before_end(constraints, self.most_recent_span);
            }
        }
    }

    /// Adds one of the following errors, selected based on constraints:
    /// - `expected ','`
    /// - `expected ')'`
    /// - `expected ',' or ')'`
    fn unexpected_token_before_end(&mut self, constraints: ArgConstraints, span: Span) {
        let error_message = match constraints {
            Optional | Required => "expected ',' or ')'",
            RequiredLast /*| OptionalLast*/ => "expected ')'",
            RequiredNotLast => "expected ','",
        };
        self.errors.add(span, error_message);
    }
}

pub enum ArgResult<'a> {
    None,
    Option(Ident, Parser<'a>),
    Struct(Parser<'a>),
}

#[derive(Clone, Copy)]
pub enum ArgConstraints {
    /// Optional:
    /// - No error for end-of-stream while looking for item.
    ///
    /// Maybe last:
    /// - `expected ',' or ')'` if unexpected tokens after item.
    Optional,

    /*
    /// Optional:
    /// - No error for end-of-stream while looking for item.
    ///
    /// Must be last:
    /// - `expected ')'` if unexpected tokens after item.
    /// - `expected ')'` if any tokens after trailing comma.
    OptionalLast,
    */
    /// Required:
    /// - Error for end-of-stream while looking for item.
    ///
    /// Maybe last:
    /// - `expected ',' or ')'` if unexpected tokens after item.
    Required,

    /// Required:
    /// - Error for end-of-stream while looking for item.
    ///
    /// Must be last:
    /// - `expected ')'` if unexpected tokens after item.
    /// - `expected ')'` if any tokens after trailing comma.
    RequiredLast,

    /// Required:
    /// - Error for end-of-stream while looking for item.
    ///
    /// Never last:
    /// - `expected ','` if unexpected tokens after item.
    RequiredNotLast,
}

enum NextHexResult {
    Digit(u32),
    Char(char),
    End,
}

fn next_hex(it: &mut str::Chars) -> NextHexResult {
    if let Some(ch) = it.next() {
        if let Some(digit) = ch.to_digit(16) {
            return NextHexResult::Digit(digit);
        } else {
            return NextHexResult::Char(ch);
        }
    } else {
        return NextHexResult::End;
    }
}

fn unescape_x(dest: &mut String, it: &mut str::Chars) -> bool {
    let mut val = 0;
    for _ in 0..2 {
        match next_hex(it) {
            NextHexResult::Digit(digit) => {
                val = (val << 4) | digit;
            }
            NextHexResult::Char(_) => {
                return false;
            }
            NextHexResult::End => {
                return false;
            }
        }
    }

    dest.push(char::from_u32(val).unwrap());
    return true;
}

fn unescape_u(dest: &mut String, it: &mut str::Chars) -> bool {
    if let Some(ch) = it.next() {
        if ch != '{' {
            return false;
        }
    } else {
        return false;
    }

    let mut val = 0;
    for n in 0..6 {
        match next_hex(it) {
            NextHexResult::Digit(digit) => {
                val = (val << 4) | digit;
            }
            NextHexResult::Char(ch) => {
                if n == 0 || ch != '}' {
                    return false;
                } else if let Some(val_ch) = char::from_u32(val) {
                    dest.push(val_ch);
                    return true; // SUCCESS
                } else {
                    return false;
                }
            }
            NextHexResult::End => {
                return false;
            }
        }
    }
    return false; // Too many digits
}

fn unescape(src: &str) -> Option<String> {
    let mut dest = String::with_capacity(src.len());
    let mut it = src.chars();
    while let Some(ch) = it.next() {
        if ch != '\\' {
            dest.push(ch);
        } else {
            match it.next() {
                Some('0') => dest.push('\0'),
                Some('n') => dest.push('\n'),
                Some('r') => dest.push('\r'),
                Some('t') => dest.push('\t'),
                Some('\\') => dest.push('\\'),
                Some('x') => {
                    if !unescape_x(&mut dest, &mut it) {
                        return None;
                    }
                }
                Some('u') => {
                    if !unescape_u(&mut dest, &mut it) {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    }

    return Some(dest);
}
