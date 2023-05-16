// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Tool that generates a markdown table describing the normal field types supported by
//! the [`write_event`] macro. This table is included in the documentation for
//! `write_event` in `tracelogging/src/lib.rs`.

#![allow(clippy::needless_return)]

use std::cmp::Ordering;

#[allow(dead_code)]
#[path = "../src/enums.rs"]
mod enums;

#[allow(dead_code)]
#[path = "../src/field_option.rs"]
mod field_option;

#[path = "../src/field_options.rs"]
mod field_options;

#[allow(dead_code)]
#[path = "../src/strings.rs"]
mod strings;

use enums::*;
use field_option::*;
use field_options::FIELD_OPTIONS;

fn main() {
    println!("/// | Field Type | Rust Type | ETW Type");
    println!("/// |------------|-----------|---------");

    let mut options = Vec::from_iter(FIELD_OPTIONS);
    options.sort_by(|a, b| option_name_cmp(a.option_name, b.option_name));
    for field in options {
        let s = field.to_markdown();
        if !s.is_empty() {
            println!("{}", &s);
        }
    }
}

/// option_name_cmp tries to be human-friendly:
/// - `foo_slice` sorts immediately after `foo`.
/// - `foo_` sorts immediately after `foo_slice` ("str" < "str_xml" < "str16").
/// - Numbers are sorted by value ("u8" < "u16").
fn option_name_cmp(val1: &str, val2: &str) -> Ordering {
    let mut v1 = val1.as_bytes();
    let mut v2 = val2.as_bytes();
    let mut pos1 = 0;
    let mut pos2 = 0;

    const SLICE: &[u8] = b"_slice";

    let slice1 = v1.ends_with(SLICE);
    if slice1 {
        v1 = &v1[..v1.len() - SLICE.len()];
    }

    let slice2 = v2.ends_with(SLICE);
    if slice2 {
        v2 = &v2[..v2.len() - SLICE.len()];
    }

    let len1 = v1.len();
    let len2 = v2.len();

    let cmp = loop {
        if pos1 == len1 {
            if pos2 == len2 {
                break slice1.cmp(&slice2);
            } else {
                break Ordering::Less;
            }
        } else if pos2 == len2 {
            break Ordering::Greater;
        }

        let ch1 = v1[pos1];
        let ch2 = v2[pos2];

        if ch1.is_ascii_digit() && ch2.is_ascii_digit() {
            let num1 = parse_int(v1, pos1);
            let num2 = parse_int(v2, pos2);

            let num_cmp = num1.value.cmp(&num2.value);
            if num_cmp != Ordering::Equal {
                break num_cmp;
            } else {
                pos1 = num1.end_pos;
                pos2 = num2.end_pos;
            }
        } else {
            let char_cmp = ch1.cmp(&ch2);
            if char_cmp != Ordering::Equal {
                if ch1 == b'_' {
                    break Ordering::Less;
                } else if ch2 == b'_' {
                    break Ordering::Greater;
                } else {
                    break char_cmp;
                }
            }

            pos1 += 1;
            pos2 += 1;
        }
    };

    return cmp;
}

trait ToMarkdown {
    fn to_markdown(&self) -> String;
    fn normal_field(&self, s: &mut String, type_path: &[&str], is_slice: bool, note: &str);
}

impl ToMarkdown for FieldOption {
    /// Produces a markdown table row that describes this field type, or "" if field type is special.
    ///
    /// `| Field Type | Rust Type | ETW Type`
    fn to_markdown(&self) -> String {
        let mut s = String::new();

        match self.strategy {
            FieldStrategy::Scalar => {
                let note = if self.option_name == "errno" {
                    "errno"
                } else {
                    ""
                };
                self.normal_field(&mut s, self.value_type, false, note);
            }
            FieldStrategy::Slice => {
                let note = if self.option_name == "errno_slice" {
                    "errno"
                } else {
                    ""
                };
                self.normal_field(&mut s, self.value_type, true, note);
            }
            FieldStrategy::SystemTime => {
                self.normal_field(&mut s, &["std", "time", "SystemTime"], false, "systemtime");
            }
            FieldStrategy::Time32 | FieldStrategy::Time64 => {
                self.normal_field(&mut s, self.value_type, false, "time");
            }
            FieldStrategy::Sid => {
                self.normal_field(&mut s, self.value_type, true, "sid");
            }
            FieldStrategy::CStr => {
                self.normal_field(&mut s, self.value_type, true, "cstr");
            }
            FieldStrategy::Counted => {
                let note = if matches!(self.intype, InType::BinaryC) {
                    "binaryc"
                } else {
                    ""
                };
                self.normal_field(&mut s, self.value_type, self.value_array_count == 0, note);
            }
            FieldStrategy::Struct
            | FieldStrategy::RawStruct
            | FieldStrategy::RawStructSlice
            | FieldStrategy::RawData
            | FieldStrategy::RawField
            | FieldStrategy::RawFieldSlice
            | FieldStrategy::RawMeta
            | FieldStrategy::RawMetaSlice => {}
        }

        return s;
    }

    fn normal_field(&self, s: &mut String, type_path: &[&str], is_slice: bool, note: &str) {
        use std::fmt::Write;

        s.push_str("/// | ");

        s.push('`');
        s.push_str(self.option_name);
        s.push('`');
        if !note.is_empty() {
            s.push_str(" [^");
            s.push_str(note);
            s.push(']');
        }

        s.push_str(" | `&");

        if is_slice {
            s.push('[');
        }

        if self.value_array_count != 0 {
            s.push('[');
        }

        let type_path_start = if type_path[0] == "core" { 2 } else { 0 };
        s.push_str(type_path[type_path_start]);
        for type_path_part in type_path.iter().skip(type_path_start + 1) {
            s.push_str("::");
            s.push_str(type_path_part);
        }

        if self.value_array_count != 0 {
            write!(s, "; {}]", self.value_array_count).unwrap();
        }

        if is_slice {
            s.push(']');
        }

        s.push_str("` | ");

        push_enum_value(s, "InType", intype_to_string(self.intype));
        if !matches!(self.outtype, OutType::Default) {
            s.push_str(" + ");
            push_enum_value(s, "OutType", outtype_to_string(self.outtype));
        }
    }
}

struct ParseResult {
    end_pos: usize,
    value: u32,
}

fn push_enum_value(s: &mut String, enum_name: &str, enum_value: &str) {
    s.push_str("[`");
    s.push_str(enum_value);
    s.push_str("`](");
    s.push_str(enum_name);
    s.push_str("::");
    s.push_str(enum_value);
    s.push(')');
}

fn parse_int(str: &[u8], pos: usize) -> ParseResult {
    let mut end_pos = pos + 1;
    let mut value = (str[pos] - b'0') as u32;
    while end_pos != str.len() && str[end_pos].is_ascii_digit() {
        value = value * 10 + (str[end_pos] - b'0') as u32;
        end_pos += 1;
    }

    return ParseResult { end_pos, value };
}

fn intype_to_string(value: InType) -> &'static str {
    return match value {
        InType::Invalid => "Invalid",
        InType::CStr16 => "CStr16",
        InType::CStr8 => "CStr8",
        InType::I8 => "I8",
        InType::U8 => "U8",
        InType::I16 => "I16",
        InType::U16 => "U16",
        InType::I32 => "I32",
        InType::U32 => "U32",
        InType::I64 => "I64",
        InType::U64 => "U64",
        InType::F32 => "F32",
        InType::F64 => "F64",
        InType::Bool32 => "Bool32",
        InType::Binary => "Binary",
        InType::Guid => "Guid",
        InType::_HexSizePlatformSpecific => "HexSizePlatformSpecific",
        InType::FileTime => "FileTime",
        InType::SystemTime => "SystemTime",
        InType::Sid => "Sid",
        InType::Hex32 => "Hex32",
        InType::Hex64 => "Hex64",
        InType::Str16 => "Str16",
        InType::Str8 => "Str8",
        InType::Struct => "Struct",
        InType::BinaryC => "BinaryC",
        InType::ISize => "ISize",
        InType::USize => "USize",
        InType::HexSize => "HexSize",
    };
}

fn outtype_to_string(value: OutType) -> &'static str {
    return match value {
        OutType::Default => "Default",
        OutType::_NoPrint => "NoPrint",
        OutType::String => "String",
        OutType::Boolean => "Boolean",
        OutType::Hex => "Hex",
        OutType::Pid => "Pid",
        OutType::Tid => "Tid",
        OutType::Port => "Port",
        OutType::IPv4 => "IPv4",
        OutType::IPv6 => "IPv6",
        OutType::SocketAddress => "SocketAddress",
        OutType::Xml => "Xml",
        OutType::Json => "Json",
        OutType::Win32Error => "Win32Error",
        OutType::NtStatus => "NtStatus",
        OutType::HResult => "HResult",
        OutType::_DateTime => "DateTime",
        OutType::_Signed => "Signed",
        OutType::_Unsigned => "Unsigned",
        OutType::_DateTimeCultureInsensitive => "DateTimeCultureInsensitive",
        OutType::Utf8 => "Utf8",
        OutType::_Pkcs7WithTypeInfo => "Pkcs7WithTypeInfo",
        OutType::CodePointer => "CodePointer",
        OutType::DateTimeUtc => "DateTimeUtc",
    };
}
