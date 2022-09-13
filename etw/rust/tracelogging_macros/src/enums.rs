// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![allow(non_upper_case_globals)]

#[derive(Clone, Copy)]
pub enum EnumToken {
    U8(u8),
    Str(&'static str),
}

#[derive(Clone, Copy)]
pub enum InType {
    Invalid = 0,
    CStr16 = 1,
    CStr8 = 2,
    I8 = 3,
    U8 = 4,
    I16 = 5,
    U16 = 6,
    I32 = 7,
    U32 = 8,
    I64 = 9,
    U64 = 10,
    F32 = 11,
    F64 = 12,
    Bool32 = 13,
    Binary = 14,
    Guid = 15,
    _HexSizePlatformSpecific = 16,
    FileTime = 17,
    SystemTime = 18,
    Sid = 19,
    Hex32 = 20,
    Hex64 = 21,
    Str16 = 22,
    Str8 = 23,
    Struct = 24,
    BinaryC = 25,

    // The following types need to be expressed symbolically in the generated code, i.e.
    // we have to generate "HexSize.to_int()" instead of literal "21".
    ISize,
    USize,
    HexSize,
}

impl InType {
    pub const VariableCountFlag: u8 = 0x40;

    pub const fn to_token(self) -> EnumToken {
        match self {
            InType::ISize => EnumToken::Str("ISize"),
            InType::USize => EnumToken::Str("USize"),
            InType::HexSize => EnumToken::Str("HexSize"),
            other => EnumToken::U8(other as u8),
        }
    }
}

#[derive(Clone, Copy)]
pub enum OutType {
    Default = 0,
    _NoPrint = 1,
    String = 2,
    Boolean = 3,
    Hex = 4,
    Pid = 5,
    Tid = 6,
    Port = 7,
    IPv4 = 8,
    IPv6 = 9,
    SocketAddress = 10,
    Xml = 11,
    Json = 12,
    Win32Error = 13,
    NtStatus = 14,
    HResult = 15,
    _DateTime = 16,
    _Signed = 17,
    _Unsigned = 18,
    _DateTimeCultureInsensitive = 33,
    Utf8 = 35,
    _Pkcs7WithTypeInfo = 36,
    CodePointer = 37,
    DateTimeUtc = 38,
}

impl OutType {
    pub const TypeMask: u8 = 0x7F;
}
