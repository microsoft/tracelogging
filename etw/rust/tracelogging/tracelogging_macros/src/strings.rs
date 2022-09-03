// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

/// Channel names special-cased by channel(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const CHANNEL_ENUMS: &[&str] = &["ProviderMetadata", "TraceClassic", "TraceLogging"];

/// Level names special-cased by level(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const LEVEL_ENUMS: &[&str] = &[
    "Critical",
    "Error",
    "Informational",
    "LogAlways",
    "Verbose",
    "Warning",
];

/// Opcode names special-cased by opcode(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const OPCODE_ENUMS: &[&str] = &[
    "DC_Start",
    "DC_Stop",
    "Extension",
    "Info",
    "Receive",
    "Reply",
    "Resume",
    "Send",
    "Start",
    "Stop",
    "Suspend",
];

/// InType names special-cased by type(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const INTYPE_ENUMS: &[&str] = &[
    "Binary",
    "Bool32",
    "CountedBinary",
    "F32",
    "F64",
    "FileTime",
    "Guid",
    "Hex32",
    "Hex64",
    "HexSize",
    "I16",
    "I32",
    "I64",
    "I8",
    "ISize",
    "Invalid",
    "Sid",
    "Str16",
    "Str8",
    "StrZ16",
    "StrZ8",
    "Struct",
    "SystemTime",
    "U16",
    "U32",
    "U64",
    "U8",
    "USize",
];

/// OutType names special-cased by format(...) option.
/// Strings must be strcmp-sorted for binary search.
pub const OUTTYPE_ENUMS: &[&str] = &[
    "Boolean",
    "CodePointer",
    "DateTime",
    "DateTimeCultureInsensitive",
    "DateTimeUtc",
    "Default",
    "HResult",
    "Hex",
    "IPv4",
    "IPv6",
    "Json",
    "NoPrint",
    "NtStatus",
    "Pid",
    "Pkcs7WithTypeInfo",
    "Port",
    "Signed",
    "SocketAddress",
    "String",
    "Tid",
    "Unsigned",
    "Utf8",
    "Win32Error",
    "Xml",
];

pub const TLG_PROV_PREFIX: &str = "_TLG_PROV_";
pub const TLG_LEVEL_CONST: &str = "_TLG_LEVEL";
pub const TLG_KEYWORD_CONST: &str = "_TLG_KEYWORD";
pub const TLG_TAG_CONST: &str = "_TLG_TAG";
pub const TLG_PROV_VAR: &str = "_tlg_prov";
pub const TLG_ARG_VAR: &str = "_tlg_arg";
pub const TLG_WRITE_FUNC: &str = "_tlg_write";
pub const TLG_META_TYPE: &str = "_TlgMeta";
pub const TLG_META_VAR: &str = "_tlg_meta";
pub const TLG_META_CONST: &str = "_TLG_META";
pub const TLG_LENGTHS_VAR: &str = "_tlg_lengths";
pub const TLG_DESC_VAR: &str = "_tlg_desc";
pub const TLG_DESC_CONST: &str = "_TLG_DESC";
pub const TLG_ACTIVITY_ID_VAR: &str = "_tlg_aid";
pub const TLG_RELATED_ID_VAR: &str = "_tlg_rid";
pub const TLG_DUR_VAR: &str = "_tlg_dur";

pub const ASREF_PATH: &[&str] = &["core", "convert", "AsRef"];
pub const IDENTITY_PATH: &[&str] = &["core", "convert", "identity"];
pub const BOOL_PATH: &[&str] = &["core", "primitive", "bool"];
pub const F32_PATH: &[&str] = &["core", "primitive", "f32"];
pub const F64_PATH: &[&str] = &["core", "primitive", "f64"];
pub const I8_PATH: &[&str] = &["core", "primitive", "i8"];
pub const I16_PATH: &[&str] = &["core", "primitive", "i16"];
pub const I32_PATH: &[&str] = &["core", "primitive", "i32"];
pub const I64_PATH: &[&str] = &["core", "primitive", "i64"];
pub const ISIZE_PATH: &[&str] = &["core", "primitive", "isize"];
pub const U8_PATH: &[&str] = &["core", "primitive", "u8"];
pub const U16_PATH: &[&str] = &["core", "primitive", "u16"];
pub const U32_PATH: &[&str] = &["core", "primitive", "u32"];
pub const U64_PATH: &[&str] = &["core", "primitive", "u64"];
pub const USIZE_PATH: &[&str] = &["core", "primitive", "usize"];
pub const ASSERT_PATH: &[&str] = &["core", "assert"];
pub const OPTION_PATH: &[&str] = &["core", "option", "Option"];
pub const OPTION_NONE_PATH: &[&str] = &["core", "option", "Option", "None"];
pub const OPTION_SOME_PATH: &[&str] = &["core", "option", "Option", "Some"];
pub const RESULT_OK_PATH: &[&str] = &["core", "result", "Result", "Ok"];
pub const RESULT_ERR_PATH: &[&str] = &["core", "result", "Result", "Err"];
pub const SYSTEMTIME_DURATION_SINCE_PATH: &[&str] =
    &["std", "time", "SystemTime", "duration_since"];
pub const SYSTEMTIME_UNIX_EPOCH_PATH: &[&str] = &["std", "time", "SystemTime", "UNIX_EPOCH"];

pub const CHANNEL_TRACELOGGING_PATH: &[&str] = &["tracelogging", "Channel", "TraceLogging"];
pub const INTYPE_PATH: &[&str] = &["tracelogging", "InType"];
pub const LEVEL_PATH: &[&str] = &["tracelogging", "Level"];
pub const LEVEL_VERBOSE_PATH: &[&str] = &["tracelogging", "Level", "Verbose"];
pub const OPCODE_INFO_PATH: &[&str] = &["tracelogging", "Opcode", "Info"];
pub const OUTTYPE_PATH: &[&str] = &["tracelogging", "OutType"];
pub const OUTTYPE_FROM_INT_PATH: &[&str] = &["tracelogging", "OutType", "from_int"];
pub const GUID_PATH: &[&str] = &["tracelogging", "Guid"];
pub const GUID_FROM_FIELDS_PATH: &[&str] = &["tracelogging", "Guid", "from_fields"];
pub const PROVIDER_PATH: &[&str] = &["tracelogging", "Provider"];

pub const PROVIDER_NEW_PATH: &[&str] = &["tracelogging", "_internal", "provider_new"];
pub const PROVIDER_WRITE_TRANSFER_PATH: &[&str] =
    &["tracelogging", "_internal", "provider_write_transfer"];
pub const META_AS_BYTES_PATH: &[&str] = &["tracelogging", "_internal", "meta_as_bytes"];
pub const TAG_ENCODE_PATH: &[&str] = &["tracelogging", "_internal", "tag_encode"];
pub const TAG_SIZE_PATH: &[&str] = &["tracelogging", "_internal", "tag_size"];
pub const COUNTED_SIZE_PATH: &[&str] = &["tracelogging", "_internal", "counted_size"];
pub const SLICE_COUNT_PATH: &[&str] = &["tracelogging", "_internal", "slice_count"];
pub const FILETIME_FROM_DURATION_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "filetime_from_duration_since_1970",
];

pub const EVENTDESC_PATH: &[&str] = &["tracelogging", "_internal", "EventDescriptor"];
pub const EVENTDESC_FROM_PARTS_PATH: &[&str] =
    &["tracelogging", "_internal", "EventDescriptor", "from_parts"];

pub const DATADESC_FROM_RAW_BYTES_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "EventDataDescriptor",
    "from_raw_bytes",
];
pub const DATADESC_FROM_VALUE_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "EventDataDescriptor",
    "from_value",
];
pub const DATADESC_FROM_SID_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "EventDataDescriptor",
    "from_sid",
];
pub const DATADESC_FROM_STRZ_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "EventDataDescriptor",
    "from_strz",
];
pub const DATADESC_FROM_SLICE_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "EventDataDescriptor",
    "from_slice",
];
pub const DATADESC_FROM_COUNTED_PATH: &[&str] = &[
    "tracelogging",
    "_internal",
    "EventDataDescriptor",
    "from_counted",
];
