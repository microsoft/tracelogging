// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use proc_macro::*;

use crate::enums::{EnumToken, InType};
use crate::expression::Expression;
use crate::field_info::FieldInfo;
use crate::field_option::{FieldOption, FieldStrategy};
use crate::ident_builder::IdentBuilder;
use crate::strings::*;
use crate::tree::Tree;

use crate::event_info::EventInfo;

pub struct EventGenerator {
    /// tokens for declaring the _TLG_TAGn constants.
    tags_tree: Tree,
    /// tokens in the _TlgMeta(...) type definition.
    meta_type_tree: Tree,
    /// tokens in the _TlgMeta(...) initializer.
    meta_init_tree: Tree,
    /// tokens in the _tlg_write(...) function signature.
    func_args_tree: Tree,
    /// tokens in the _tlg_write(...) function call.
    func_call_tree: Tree,
    /// tokens in the _tlg_lengths = [...] array initializer.
    lengths_init_tree: Tree,
    /// tokens in the EventDataDescriptor &[...] array initializer.
    data_desc_init_tree: Tree,
    /// Code that runs if the provider is enabled.
    enabled_tree: Tree,
    /// scratch tree 1
    tree1: Tree,
    /// scratch tree 2
    tree2: Tree,
    /// scratch tree 3
    tree3: Tree,
    /// "_TLG_TAGn"
    tag_n: IdentBuilder,
    /// "_tlg_argN"
    arg_n: IdentBuilder,
    /// Buffered _TlgMeta bytes.
    meta_buffer: Vec<u8>,
    /// number of fields added so far
    field_count: u16,
    /// number of runtime lengths needed
    lengths_count: u16,
}

impl EventGenerator {
    pub fn new(span: Span) -> Self {
        return Self {
            tags_tree: Tree::new(span),
            meta_type_tree: Tree::new(span),
            meta_init_tree: Tree::new(span),
            func_args_tree: Tree::new(span),
            func_call_tree: Tree::new(span),
            lengths_init_tree: Tree::new(span),
            data_desc_init_tree: Tree::new(span),
            enabled_tree: Tree::new(span),
            tree1: Tree::new(span),
            tree2: Tree::new(span),
            tree3: Tree::new(span),
            tag_n: IdentBuilder::new(TLG_TAG_CONST),
            arg_n: IdentBuilder::new(TLG_ARG_VAR),
            meta_buffer: Vec::with_capacity(128),
            field_count: 0,
            lengths_count: 0,
        };
    }

    pub fn generate(&mut self, mut event: EventInfo) -> TokenStream {
        self.meta_buffer.clear();
        self.field_count = 0;
        self.lengths_count = 0;

        // Before-field stuff:

        // metadata size: u16 = size_of::<_TlgMeta>() as u16
        self.meta_type_tree.add_path(U16_PATH);
        self.meta_init_tree
            .add_sizeof(TLG_META_TYPE)
            .add_ident("as")
            .add_path(U16_PATH);

        // event tag
        self.tag_n.clear_suffix();
        self.add_tag(event.tag);

        // event name
        self.meta_buffer.extend(event.name.as_bytes());
        self.meta_buffer.push(0);

        // data descriptors for provider metadata and event metadata
        const EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA: u32 = 2;
        const EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA: u32 = 1;
        self.data_desc_init_tree
            // ::tracelogging::_internal::EventDataDescriptor::from_raw_slice(prov_meta),
            .add_path_call(
                DATADESC_FROM_RAW_BYTES_PATH,
                self.tree1
                    // _tlg_prov.raw_meta(), 2
                    .add_ident(TLG_PROV_VAR)
                    .add_punct(".")
                    .add_ident("raw_meta")
                    .add_group_paren([])
                    .add_punct(",")
                    .add_literal(Literal::u32_unsuffixed(
                        EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA,
                    ))
                    .drain(),
            )
            .add_punct(",")
            // ::tracelogging::_internal::EventDataDescriptor::from_raw_slice(event_meta),
            .add_path_call(
                DATADESC_FROM_RAW_BYTES_PATH,
                self.tree1
                    // _tlg_meta, 1
                    .add_ident(TLG_META_VAR)
                    .add_punct(",")
                    .add_literal(Literal::u32_unsuffixed(
                        EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA,
                    ))
                    .drain(),
            )
            .add_punct(",");

        // always-present args for the helper function's prototype
        self.func_args_tree
            // _tlg_prov: &tlg::Provider
            .add_ident(TLG_PROV_VAR)
            .add_punct(":")
            .add_punct("&")
            .add_path(PROVIDER_PATH)
            // , _tlg_meta: &[u8]
            .add_punct(",")
            .add_ident(TLG_META_VAR)
            .add_punct(":")
            .add_punct("&")
            .add_group_square(self.tree1.add_path(U8_PATH).drain())
            // , _tlg_desc: &tlg::EventDescriptor
            .add_punct(",")
            .add_ident(TLG_DESC_VAR)
            .add_punct(":")
            .add_punct("&")
            .add_path(EVENTDESC_PATH)
            // , activity_id: Option<&[u8; 16]>
            .add_punct(",")
            .add_ident(TLG_ACTIVITY_ID_VAR)
            .add_punct(":")
            .add_path(OPTION_PATH)
            .add_punct("<")
            .add_punct("&")
            .add_group_square(
                self.tree1
                    .add_path(U8_PATH)
                    .add_punct(";")
                    .add_literal(Literal::usize_unsuffixed(16))
                    .drain(),
            )
            .add_punct(">")
            // , related_id: Option<&[u8; 16]>
            .add_punct(",")
            .add_ident(TLG_RELATED_ID_VAR)
            .add_punct(":")
            .add_path(OPTION_PATH)
            .add_punct("<")
            .add_punct("&")
            .add_group_square(
                self.tree1
                    .add_path(U8_PATH)
                    .add_punct(";")
                    .add_literal(Literal::usize_unsuffixed(16))
                    .drain(),
            )
            .add_punct(">");

        // always-present args for the helper function's call site
        self.func_call_tree
            // &PROVIDER
            .add_punct("&")
            .add_token(event.provider_symbol.clone())
            // , tlg::meta_as_bytes(&_tlg_meta)
            .add_punct(",")
            .add_path_call(
                META_AS_BYTES_PATH,
                self.tree1.add_punct("&").add_ident(TLG_META_CONST).drain(),
            )
            // , &_TLG_DESC
            .add_punct(",")
            .add_punct("&")
            .add_ident(TLG_DESC_CONST)
            // , None-or-Some(borrow(activity_id_tokens...))
            .add_punct(",")
            .push_span(event.activity_id.context)
            .add_borrowed_option_from_tokens(&mut self.tree1, event.activity_id.tokens)
            .pop_span()
            // , None-or-Some(borrow(related_id_tokens...))
            .add_punct(",")
            .push_span(event.related_id.context)
            .add_borrowed_option_from_tokens(&mut self.tree1, event.related_id.tokens)
            .pop_span();

        // Add the per-field stuff:

        for field in event.fields.drain(..) {
            self.add_field(field);
        }

        self.flush_meta_buffer();

        // code that runs if the provider is enabled:
        /*
        const _TLG_DESC = EventDescriptor::from_raw_parts(...);
        tags_tree...
        struct _TlgMeta(meta_type_tree...);
        const _TLG_META = _TlgMeta(meta_init_tree...);
        fn _tlg_write(func_args_tree...) -> u32 {
            let _tlg_lengths = [lengths_init_tree...];
            provider_write_transfer(prov, desc, aid, rid, &[data_desc_init_tree...]);
        }
        _tlg_write(func_call_tree)
        */

        self.enabled_tree
            // const _TLG_DESC: EventDescriptor = EventDescriptor::from_raw_parts(...);
            .add_const_from_tokens(
                TLG_DESC_CONST,
                EVENTDESC_PATH,
                self.tree1
                    .add_path_call(
                        EVENTDESC_FROM_PARTS_PATH,
                        self.tree2
                            .add_tokens(event.id_tokens)
                            .add_punct(",")
                            .add_tokens(event.version_tokens)
                            .add_punct(",")
                            .add_tokens(event.channel_tokens)
                            .add_punct(",")
                            .add_ident(TLG_LEVEL_CONST)
                            .add_punct(",")
                            .add_tokens(event.opcode_tokens)
                            .add_punct(",")
                            .add_tokens(event.task_tokens)
                            .add_punct(",")
                            .add_ident(TLG_KEYWORD_CONST)
                            .drain(),
                    )
                    .drain(),
            )
            // const _TLG_TAG: u32 = EVENT_TAG; const _TLG_TAG3: u32 = FIELD3_TAG;
            .add_tokens(self.tags_tree.drain())
            // #[repr(C, packed)]
            .add_outer_attribute(
                "repr",
                self.tree1
                    .add_ident("C")
                    .add_punct(",")
                    .add_ident("packed")
                    .drain(),
            )
            // struct _TlgMeta(meta_types...);
            .add_ident("struct")
            .add_ident(TLG_META_TYPE)
            .add_group_paren(self.meta_type_tree.drain())
            .add_punct(";")
            // const _TLG_META: _TlgMeta = _TlgMeta(...);
            .add_ident("const")
            .add_ident(TLG_META_CONST)
            .add_punct(":")
            .add_ident(TLG_META_TYPE)
            .add_punct("=")
            .add_ident(TLG_META_TYPE)
            .add_group_paren(self.meta_init_tree.drain())
            .add_punct(";")
            // #[allow(clippy::too_many_arguments)]
            .add_outer_attribute(
                "allow",
                self.tree1
                    .add_ident("clippy")
                    .add_punct("::")
                    .add_ident("too_many_arguments")
                    .drain(),
            )
            // Make a helper function and then call it. This does the following:
            // - Keep temporaries alive (this could also be done with a match expression).
            // - Give the optimizer the option to merge identical helpers.
            // fn _tlg_write(prov, meta, desc, aid, rid, args...) -> { ... }
            .add_ident("fn")
            .add_ident(TLG_WRITE_FUNC)
            .add_group_paren(self.func_args_tree.drain())
            .add_punct("->")
            .add_path(U32_PATH)
            .add_group_curly(
                self.tree1
                    // let _tlg_lengths = [...];
                    .add_ident("let")
                    .add_ident(TLG_LENGTHS_VAR)
                    .add_punct(":")
                    .add_group_square(
                        self.tree2
                            .add_path(U16_PATH)
                            .add_punct(";")
                            .add_literal(Literal::u16_unsuffixed(self.lengths_count))
                            .drain(),
                    )
                    .add_punct("=")
                    .add_group_square(self.lengths_init_tree.drain())
                    .add_punct(";")
                    // provider_write_transfer(_tlg_prov, meta, &_TLG_DESC, activity_id, related_id, &[data...])
                    .add_path_call(
                        PROVIDER_WRITE_TRANSFER_PATH,
                        self.tree2
                            .add_ident(TLG_PROV_VAR)
                            .add_punct(",")
                            .add_ident(TLG_DESC_VAR) // descriptor
                            .add_punct(",")
                            .add_ident(TLG_ACTIVITY_ID_VAR)
                            .add_punct(",")
                            .add_ident(TLG_RELATED_ID_VAR)
                            .add_punct(",")
                            .add_punct("&")
                            .add_group_square(self.data_desc_init_tree.drain())
                            .drain(),
                    )
                    .drain(),
            )
            // _tlg_write(prov, meta, aid, rid, values...)
            .add_ident(TLG_WRITE_FUNC)
            .add_group_paren(self.func_call_tree.drain());

        // put it all together:
        /*
        const _TLG_KEYWORD = keywords...;
        const _TLG_LEVEL = level...;
        if(!tlg_prov_var.enabled(_TLG_LEVEL, _TLG_KEYWORD)) {
            0
        } else {
            enabled_tree...
        }
        */

        let event_tree = &mut self.tree2; // Alias tree2 to save a tree.

        // _TLG_KEYWORD
        if event.keywords.len() == 1 {
            // Generate simple output if only one keyword.
            // const _TLG_KEYWORD: u64 = KEYWORDS[0];
            let keyword = event.keywords.pop().unwrap();
            event_tree
                .push_span(keyword.context)
                .add_const_from_tokens(TLG_KEYWORD_CONST, U64_PATH, keyword.tokens)
                .pop_span();
        } else {
            // More-complex output needed in other cases.
            //
            // We have suboptimal results if we combine the subexpressions ourselves,
            // e.g. doing "const X = (KEYWORDS0) | (KEYWORDS1);"" would result in
            // suboptimal error reporting for syntax errors in the user-supplied
            // expressions as well as warnings for unnecessary parentheses. Instead,
            // evaluate the subexpressions separately then combine the resulting
            // constants. This works for any number of keywords.
            //
            // const _TLG_KEYWORD0: u64 = KEYWORDS0;
            // const _TLG_KEYWORD1: u64 = KEYWORDS1;
            // const _TLG_KEYWORD: u64 = 0u64 | _TLG_KEYWORD0 | _TLG_KEYWORD1;

            let mut tlg_keyword_n = IdentBuilder::new(TLG_KEYWORD_CONST);

            // Build up "const _TLG_KEYWORDn: u64 = KEYWORDSn; ..."" in event_tree.
            // Build up "0u64 | _TLG_KEYWORD0 | _TLG_KEYWORD1 ..." in tree1.

            // tree1 += "0u64"
            self.tree1.add_literal(Literal::u64_suffixed(0));

            for (n, keyword) in event.keywords.drain(..).enumerate() {
                // event_tree += "const _TLG_KEYWORDn: u64 = KEYWORDSn;"
                event_tree
                    .push_span(keyword.context)
                    .add_const_from_tokens(tlg_keyword_n.set_suffix(n), U64_PATH, keyword.tokens)
                    .pop_span();

                // tree1 += "| _TLG_KEYWORDn"
                self.tree1.add_punct("|").add_ident(tlg_keyword_n.current());
            }

            // event_tree += "const _TLG_KEYWORD: u64 = 0u64 | _TLG_KEYWORD0 | _TLG_KEYWORD1;"
            event_tree.add_const_from_tokens(TLG_KEYWORD_CONST, U64_PATH, self.tree1.drain());
        }

        event_tree
            // const _TLG_LEVEL: Level = LEVEL;
            .push_span(event.level.context)
            .add_const_from_tokens(TLG_LEVEL_CONST, LEVEL_PATH, event.level.tokens)
            .pop_span()
            // if !PROVIDER.enabled(_TLG_LEVEL, _TLG_KEYWORD) { 0 }
            .add_ident("if")
            .add_punct("!")
            .add_token(event.provider_symbol)
            .add_punct(".")
            .add_ident("enabled")
            .add_group_paren(
                self.tree1
                    .add_ident(TLG_LEVEL_CONST)
                    .add_punct(",")
                    .add_ident(TLG_KEYWORD_CONST)
                    .drain(),
            )
            .add_group_curly(self.tree1.add_literal(Literal::u32_suffixed(0)).drain())
            // else { enabled_tree... }
            .add_ident("else")
            .add_group_curly(self.enabled_tree.drain());

        // Wrap the event in "{...}":
        let event_tokens = TokenStream::from(TokenTree::Group(Group::new(
            Delimiter::Brace,
            event_tree.drain().collect(),
        )));

        if event.debug {
            println!("{}", event_tokens);
        }

        return event_tokens;
    }

    fn add_field(&mut self, field: FieldInfo) {
        // Metadata

        if field.option.strategy.has_metadata() {
            self.meta_buffer.extend(field.name.as_bytes());
            self.meta_buffer.push(0);

            let has_out = !field.outtype_or_field_count_expr.is_empty()
                || field.outtype_or_field_count_int != 0;
            let has_tag = !field.tag.is_empty();

            let inflags = (if has_out || has_tag { 0x80 } else { 0 })
                | (if field.option.strategy.is_slice() {
                    InType::VariableCountFlag
                } else {
                    0
                });
            self.add_typecode_meta(
                INTYPE_PATH,
                field.intype_tokens,
                field.type_name_span,
                field.option.intype.to_token(),
                inflags,
            );

            if has_out || has_tag {
                let outflags = if has_tag { 0x80 } else { 0 };
                self.add_typecode_meta(
                    OUTTYPE_PATH,
                    field.outtype_or_field_count_expr.tokens,
                    field.outtype_or_field_count_expr.context,
                    EnumToken::U8(field.outtype_or_field_count_int),
                    outflags,
                );
            }

            if has_tag {
                self.tag_n.set_suffix(self.field_count as usize);
                self.add_tag(field.tag);
            }
        }

        // Data

        self.data_desc_init_tree.push_span(field.type_name_span);
        self.arg_n.set_suffix(self.field_count as usize);

        match field.option.strategy {
            FieldStrategy::Scalar => {
                self.tree1
                    // , identity::<&VALUE_TYPE>(value_tokens...)
                    .push_span(field.type_name_span) // Use identity(...) as a target for error messages.
                    .add_identity_call(
                        &mut self.tree2,
                        field.option.value_type,
                        field.option.value_array_count,
                        field.value_tokens,
                    )
                    .pop_span();

                // Prototype: , _tlg_argN: &value_type
                // Call site: , identity::<&value_type>(value_tokens...)
                self.add_func_scalar_arg(field.option); // consumes tree1

                // EventDataDescriptor::from_value(_tlg_argN),
                self.add_data_desc_for_arg_n(DATADESC_FROM_VALUE_PATH);
            }

            FieldStrategy::Time32 | FieldStrategy::Time64 => {
                let filetime_from_time_path = if let FieldStrategy::Time64 = field.option.strategy {
                    FILETIME_FROM_TIME64_PATH
                } else {
                    FILETIME_FROM_TIME32_PATH
                };

                self.tree1
                    // , &filetime_from_timeNN(value_tokens...)
                    .push_span(field.type_name_span) // Use filetime_from_timeNN(...) as a target for error messages.
                    .add_punct("&")
                    .add_path_call(filetime_from_time_path, field.value_tokens)
                    .pop_span();

                // Prototype: , _tlg_argN: &value_type
                // Call site: , &filetime_from_timeNN(value_tokens...)
                self.add_func_scalar_arg(field.option); // consumes tree1

                // EventDataDescriptor::from_value(_tlg_argN),
                self.add_data_desc_for_arg_n(DATADESC_FROM_VALUE_PATH);
            }

            FieldStrategy::SystemTime => {
                self.tree1
                    // match SystemTime::duration_since(value_tokens, SystemTime::UNIX_EPOCH) { ... }
                    .push_span(field.type_name_span) // Use duration_since(...) as a target for error messages.
                    .add_punct("&")
                    .add_ident("match")
                    .add_path_call(
                        SYSTEMTIME_DURATION_SINCE_PATH,
                        self.tree2
                            .add_tokens(field.value_tokens)
                            .add_punct(",")
                            .add_path(SYSTEMTIME_UNIX_EPOCH_PATH)
                            .drain(),
                    )
                    .add_group_curly(
                        self.tree2
                            // Ok(_tlg_dur) => filetime_from_duration_after_1970(_tlg_dur),
                            .add_path(RESULT_OK_PATH)
                            .add_group_paren(self.tree3.add_ident(TLG_DUR_VAR).drain())
                            .add_punct("=>")
                            .add_path_call(
                                FILETIME_FROM_DURATION_AFTER_PATH,
                                self.tree3.add_ident(TLG_DUR_VAR).drain(),
                            )
                            .add_punct(",")
                            // Err(_tlg_dur) => filetime_from_duration_before_1970(_tlg_dur.duration()),
                            .add_path(RESULT_ERR_PATH)
                            .add_group_paren(self.tree3.add_ident(TLG_DUR_VAR).drain())
                            .add_punct("=>")
                            .add_path_call(
                                FILETIME_FROM_DURATION_BEFORE_PATH,
                                self.tree3
                                    .add_ident(TLG_DUR_VAR)
                                    .add_punct(".")
                                    .add_ident("duration")
                                    .add_group_paren([])
                                    .drain(),
                            )
                            .add_punct(",")
                            .drain(),
                    )
                    .pop_span();

                // Prototype: , _tlg_argN: &i64
                // Call site: , &match SystemTime::duration_since(value_tokens, SystemTime::UNIX_EPOCH) { ... }
                self.add_func_scalar_arg(field.option); // consumes tree1

                // EventDataDescriptor::from_value(_tlg_argN),
                self.add_data_desc_for_arg_n(DATADESC_FROM_VALUE_PATH);
            }

            FieldStrategy::RawData | FieldStrategy::RawField | FieldStrategy::RawFieldSlice => {
                // Prototype: , _tlg_argN: &[value_type]
                // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                self.add_func_slice_arg(field.option, field.type_name_span, field.value_tokens);

                // EventDataDescriptor::from_counted(_tlg_argN),
                self.add_data_desc_for_arg_n(DATADESC_FROM_COUNTED_PATH);
            }

            FieldStrategy::Sid => {
                // Prototype: , _tlg_argN: &[value_type]
                // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                self.add_func_slice_arg(field.option, field.type_name_span, field.value_tokens);

                // EventDataDescriptor::from_sid(_tlg_argN),
                self.add_data_desc_for_arg_n(DATADESC_FROM_SID_PATH);
            }

            FieldStrategy::CStr => {
                // Prototype: , _tlg_argN: &[value_type]
                // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                self.add_func_slice_arg(field.option, field.type_name_span, field.value_tokens);

                // EventDataDescriptor::from_cstr(_tlg_argN),
                self.add_data_desc_for_arg_n(DATADESC_FROM_CSTR_PATH);

                self.data_desc_init_tree
                    // EventDataDescriptor::from_value<value_type>(&0),
                    .add_path(DATADESC_FROM_VALUE_PATH)
                    .add_punct("::")
                    .add_punct("<")
                    .add_path(field.option.value_type) // value_type is u8 or u16
                    .add_punct(">")
                    .add_group_paren(
                        self.tree1
                            .add_punct("&")
                            .add_literal(Literal::u8_unsuffixed(0))
                            .drain(),
                    )
                    .add_punct(",");
            }

            FieldStrategy::Counted => {
                if field.option.value_array_count == 0 {
                    // Prototype: , _tlg_argN: &[value_type]
                    // Call site: , AsRef::<[value_type]>::as_ref(value_tokens...)
                    self.add_func_slice_arg(field.option, field.type_name_span, field.value_tokens);
                } else {
                    // e.g. ipv6 takes a fixed-length array, not a variable-length slice
                    self.tree1
                        // , identity::<&value_type>(value_tokens...)
                        .push_span(field.type_name_span) // Use identity(...) as a target for error messages.
                        .add_identity_call(
                            &mut self.tree2,
                            field.option.value_type,
                            field.option.value_array_count,
                            field.value_tokens,
                        )
                        .pop_span();

                    // Prototype: , _tlg_argN: &[value_type; value_array_count]
                    // Call site: , identity::<&[value_type; value_array_count]>(value_tokens...)
                    self.add_func_scalar_arg(field.option); // consumes tree1
                }

                // EventDataDescriptor::from_value(&_tlg_lengths[N]),
                // EventDataDescriptor::from_counted(_tlg_argN),
                self.add_data_desc_with_length(COUNTED_SIZE_PATH, DATADESC_FROM_COUNTED_PATH);
            }

            FieldStrategy::Slice => {
                self.add_func_slice_arg(field.option, field.type_name_span, field.value_tokens);

                // EventDataDescriptor::from_value(&_tlg_lengths[N]),
                // EventDataDescriptor::from_slice(_tlg_argN),
                self.add_data_desc_with_length(SLICE_COUNT_PATH, DATADESC_FROM_SLICE_PATH);
            }

            FieldStrategy::Struct
            | FieldStrategy::RawStruct
            | FieldStrategy::RawStructSlice
            | FieldStrategy::RawMeta
            | FieldStrategy::RawMetaSlice => {}
        }

        self.data_desc_init_tree.pop_span();

        // Common

        self.field_count += 1;
    }

    fn add_data_desc_for_arg_n(&mut self, new_desc_path: &[&str]) {
        self.data_desc_init_tree
            // EventDataDescriptor::new_desc_path(_tlg_argN),
            .add_path_call(
                new_desc_path,
                self.tree1.add_ident(self.arg_n.current()).drain(),
            )
            .add_punct(",");
    }

    fn add_data_desc_with_length(&mut self, get_length_path: &[&str], new_desc_path: &[&str]) {
        // get_length_path(_tlg_argN),
        self.lengths_init_tree
            .add_path_call(
                get_length_path,
                self.tree1.add_ident(self.arg_n.current()).drain(),
            )
            .add_punct(",");

        // EventDataDescriptor::from_value(&_tlg_lengths[N]),
        // EventDataDescriptor::new_desc_path(_tlg_argN),
        self.data_desc_init_tree
            .add_path_call(
                DATADESC_FROM_VALUE_PATH,
                self.tree1
                    .add_punct("&")
                    .add_ident(TLG_LENGTHS_VAR)
                    .add_group_square(
                        self.tree2
                            .add_literal(Literal::u16_unsuffixed(self.lengths_count))
                            .drain(),
                    )
                    .drain(),
            )
            .add_punct(",");
        self.add_data_desc_for_arg_n(new_desc_path);

        self.lengths_count += 1;
    }

    // We wrap all input expressions in adapter<T>(expression) because it allows
    // us to get MUCH better error messages. We attribute the adapter<T>() tokens
    // to the type_name_span so that if the expression is the wrong type, the
    // error message says "your expression didn't match the type expected by -->"
    // and the arrow points at the type_name, which is great. In cases where
    // as_ref() can be used, we use as_ref() as the adapter. Otherwise, we use
    // identity().

    /// Prototype: , _tlg_argN: &VALUE_TYPE
    /// Call site: , tree1_tokens...
    fn add_func_scalar_arg(&mut self, field_option: &FieldOption) {
        // , _tlg_argN: &VALUE_TYPE
        self.func_args_tree
            .add_punct(",")
            .add_ident(self.arg_n.current())
            .add_punct(":")
            .add_punct("&")
            .add_scalar_type_path(
                &mut self.tree2,
                field_option.value_type,
                field_option.value_array_count,
            );

        // We do not apply AsRef for non-slice types. AsRef provides a no-op mapping
        // for slices (i.e. AsRef<[u8]>::as_ref(&u8_slice) returns &u8_slice), but
        // there is not a no-op mapping for non-slice types (i.e.
        // AsRef<u8>::as_ref(&u8_val) will be a compile error). While this is a bit
        // inconsistent, I don't think it's a problem in practice. The non-slice
        // types don't get much value from as_ref. Most of their needs are handled
        // by the Deref trait, which the compiler applies automatically.

        // , value_tokens...
        self.func_call_tree
            .add_punct(",")
            .add_tokens(self.tree1.drain());
    }

    /// Prototype: , _tlg_argN: &[VALUE_TYPE]
    /// Call site: , AsRef::<[VALUE_TYPE]>::as_ref(value_tokens...)
    fn add_func_slice_arg(
        &mut self,
        field_option: &FieldOption,
        field_type_name_span: Span,
        field_value_tokens: TokenStream,
    ) {
        // , _tlg_argN: &[VALUE_TYPE]
        self.func_args_tree
            .add_punct(",")
            .add_ident(self.arg_n.current())
            .add_punct(":")
            .add_punct("&")
            .add_group_square(
                self.tree1
                    .add_scalar_type_path(
                        &mut self.tree2,
                        field_option.value_type,
                        field_option.value_array_count,
                    )
                    .drain(),
            );

        // For cases where the expected input is a slice &[T], we apply the
        // core::convert::AsRef<[T]> trait to unwrap the provided value. This is
        // most important for strings because otherwise the str functions would only
        // accept &[u8] (they wouldn't be able to accept &str or &String). This also
        // applies to 3rd-party types, e.g. widestring's U16String implements
        // AsRef<[u16]> so it just works as a value for the str16 field types.

        // , AsRef::<[VALUE_TYPE]>::as_ref(value_tokens...)
        self.func_call_tree
            .add_punct(",")
            .push_span(field_type_name_span) // Use as_ref(...) as a target for error messages.
            .add_path(ASREF_PATH)
            .add_punct("::")
            .add_punct("<")
            .add_group_square(
                self.tree1
                    .add_scalar_type_path(
                        &mut self.tree2,
                        field_option.value_type,
                        field_option.value_array_count,
                    )
                    .drain(),
            )
            .add_punct(">")
            .add_punct("::")
            .add_ident("as_ref")
            .add_group_paren(field_value_tokens)
            .pop_span();
    }

    fn add_typecode_meta(
        &mut self,
        enum_type_path: &[&str],
        tokens: TokenStream,
        span: Span,
        type_token: EnumToken,
        flags: u8,
    ) {
        if tokens.is_empty() {
            match type_token {
                EnumToken::U8(enum_int) => {
                    self.meta_buffer.push(enum_int | flags);
                    return;
                }

                EnumToken::Str(enum_name) => {
                    self.flush_meta_buffer();

                    // , EnumVal
                    self.meta_init_tree
                        .add_punct(",")
                        .push_span(span)
                        .add_path(enum_type_path)
                        .add_punct("::")
                        .add_ident(enum_name);
                }
            };
        } else {
            self.flush_meta_buffer();

            // , identity::<EnumType>(...)
            self.meta_init_tree
                .add_punct(",")
                .push_span(span)
                .add_path(IDENTITY_PATH) // Use identity(...) as a target for error messages.
                .add_punct("::")
                .add_punct("<")
                .add_path(enum_type_path)
                .add_punct(">")
                .add_group_paren(tokens);
        }

        // , u8
        self.meta_type_tree.add_punct(",").add_path(U8_PATH);

        // .as_int()
        self.meta_init_tree
            .add_punct(".")
            .add_ident("as_int")
            .add_group_paren([]);

        // | flags
        if flags != 0 {
            self.meta_init_tree
                .add_punct("|")
                .add_literal(Literal::u8_unsuffixed(flags));
        }

        self.meta_init_tree.pop_span();
    }

    fn add_tag(&mut self, expression: Expression) {
        // Implicitly uses self.tag_const as the name for the tag's constant.

        self.flush_meta_buffer();

        // const _TLG_TAGn: u32 = TAG;
        self.tags_tree
            .push_span(expression.context)
            .add_const_from_tokens(self.tag_n.current(), U32_PATH, expression.tokens)
            // #[allow(clippy::assertions_on_constants)]
            .add_outer_attribute(
                "allow",
                self.tree1
                    .push_span(expression.context)
                    .add_ident("clippy")
                    .add_punct("::")
                    .add_ident("assertions_on_constants")
                    .pop_span()
                    .drain(),
            )
            // const _: () = assert!(_TLG_TAGn <= 0x0FFFFFFF, "...");
            .add_ident("const")
            .add_ident("_")
            .add_punct(":")
            .add_group_paren([])
            .add_punct("=")
            .add_path(ASSERT_PATH)
            .add_punct("!")
            .add_group_paren(
                self.tree1
                    .push_span(expression.context)
                    .add_ident(self.tag_n.current())
                    .add_punct("<=")
                    .add_literal(Literal::u32_unsuffixed(0x0FFFFFFF))
                    .add_punct(",")
                    .add_literal(Literal::string("tag must not be greater than 0x0FFFFFFF"))
                    .pop_span()
                    .drain(),
            )
            .add_punct(";")
            .pop_span();

        // , [u8; tag_size(_TLG_TAGn)]
        self.meta_type_tree.add_punct(",").add_group_square(
            self.tree1
                .add_path(U8_PATH)
                .add_punct(";")
                .add_path_call(
                    TAG_SIZE_PATH,
                    self.tree2
                        .push_span(expression.context)
                        .add_ident(self.tag_n.current())
                        .pop_span()
                        .drain(),
                )
                .drain(),
        );

        // , tag_encode(_TLG_TAGn)
        self.meta_init_tree.add_punct(",").add_path_call(
            TAG_ENCODE_PATH,
            self.tree1.add_ident(self.tag_n.current()).drain(),
        );
    }

    /// If `meta_buffer` is empty, does nothing, otherwise, if there are `N` bytes of
    /// metadata in meta_buffer, adds a `[u8;N]` field to `meta_type_tree`, adds a binary
    /// literal containing the data to `meta_init_tree`, then clears `meta_buffer`.
    fn flush_meta_buffer(&mut self) {
        if !self.meta_buffer.is_empty() {
            // [u8; LEN] = , b"VAL"
            self.meta_type_tree.add_punct(",").add_group_square(
                self.tree1
                    .add_path(U8_PATH)
                    .add_punct(";")
                    .add_literal(Literal::usize_unsuffixed(self.meta_buffer.len()))
                    .drain(),
            );
            self.meta_init_tree
                .add_punct(",")
                .add_punct("*")
                .add_literal(Literal::byte_string(&self.meta_buffer[..]));
            self.meta_buffer.clear();
        }
    }
}
