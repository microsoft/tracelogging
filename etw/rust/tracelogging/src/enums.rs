#![allow(non_upper_case_globals)]

use core::fmt;
use core::mem::size_of;

/// Indicates routing and decoding for an event.
/// This should almost always be set to TraceLogging (11) for TraceLogging events.
/// This should almost always be set to TraceClassic (0) for non-TraceLogging events.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Channel(u8);

impl Channel {
    /// Returns a channel with the specified value.
    #[inline(always)]
    pub const fn from_int(value: u8) -> Channel {
        return Channel(value);
    }

    /// Returns the integer value of this channel.
    #[inline(always)]
    pub const fn as_int(self) -> u8 {
        return self.0;
    }

    /// Channel for non-TraceLogging events.
    pub const TraceClassic: Channel = Channel(0);
    /// Channel for TraceLogging events.
    pub const TraceLogging: Channel = Channel(11);
    /// Channel for events from machine-generated manifests.
    pub const ProviderMetadata: Channel = Channel(12);
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return self.0.fmt(f);
    }
}

impl From<u8> for Channel {
    fn from(val: u8) -> Self {
        return Self(val);
    }
}

impl From<Channel> for u8 {
    fn from(val: Channel) -> Self {
        return val.0;
    }
}

/// Indicates the severity of an event. Use Verbose if unsure.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Level(pub(crate) u8);

impl Level {
    /// Returns a level with the specified value.
    #[inline(always)]
    pub const fn from_int(value: u8) -> Level {
        return Level(value);
    }

    /// Returns the integer value of this level.
    #[inline(always)]
    pub const fn as_int(self) -> u8 {
        return self.0;
    }

    /// Event ignores level-based filtering. This level should almost never be used.
    pub const LogAlways: Level = Level(0);
    /// Critical error event.
    pub const Critical: Level = Level(1);
    /// Error event.
    pub const Error: Level = Level(2);
    /// Warning event.
    pub const Warning: Level = Level(3);
    /// Informational event.
    pub const Informational: Level = Level(4);
    /// Verbose event.
    pub const Verbose: Level = Level(5);
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return self.0.fmt(f);
    }
}

impl From<u8> for Level {
    fn from(val: u8) -> Self {
        return Self(val);
    }
}

impl From<Level> for u8 {
    fn from(val: Level) -> Self {
        return val.0;
    }
}

/// Indicates special semantics to be used by the event decoder
/// for grouping and organizing events. For example, the Start opcode indicates the
/// beginning of an activity and the Stop opcode indicates the end of an activity. Most
/// events default to Info (0).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Opcode(u8);

impl Opcode {
    /// Returns an opcode with the specified value.
    #[inline(always)]
    pub const fn from_int(value: u8) -> Opcode {
        return Opcode(value);
    }

    /// Returns the integer value of this opcode.
    #[inline(always)]
    pub const fn as_int(self) -> u8 {
        return self.0;
    }

    /// Normal event. May have activity_id set if it is part of an activity.
    pub const Info: Opcode = Opcode(0);
    /// Event indicates the beginning of an activity. The event should set
    /// related_id to the id of the parent activity and set activity_id to the
    /// id of the newly-started activity. All subsequent events that use the
    /// new activity_id will be considered as part of this activity, up to the
    /// corresponding Stop event.
    pub const Start: Opcode = Opcode(1);
    /// Event indicates the end of an activity. The event should set activity_id
    /// to the id of the activity that is ending and should use the same level
    /// and keyword as used for the corresponding Start event.
    pub const Stop: Opcode = Opcode(2);
    /// Data Collection Start event
    pub const DC_Start: Opcode = Opcode(3);
    /// Data Collection Stop event
    pub const DC_Stop: Opcode = Opcode(4);
    /// Extension event
    pub const Extension: Opcode = Opcode(5);
    /// Reply event
    pub const Reply: Opcode = Opcode(6);
    /// Resume event
    pub const Resume: Opcode = Opcode(7);
    /// Suspend event
    pub const Suspend: Opcode = Opcode(8);
    /// Message Send event
    pub const Send: Opcode = Opcode(9);
    /// Message Receive event
    pub const Receive: Opcode = Opcode(240);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode241: Opcode = Opcode(241);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode242: Opcode = Opcode(242);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode243: Opcode = Opcode(243);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode244: Opcode = Opcode(244);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode245: Opcode = Opcode(245);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode246: Opcode = Opcode(246);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode247: Opcode = Opcode(247);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode248: Opcode = Opcode(248);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode249: Opcode = Opcode(249);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode250: Opcode = Opcode(250);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode251: Opcode = Opcode(251);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode252: Opcode = Opcode(252);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode253: Opcode = Opcode(253);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode254: Opcode = Opcode(254);
    /// Reserved for future definition by Microsoft
    pub const ReservedOpcode255: Opcode = Opcode(255);
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return self.0.fmt(f);
    }
}

impl From<u8> for Opcode {
    fn from(val: u8) -> Self {
        return Self(val);
    }
}

impl From<Opcode> for u8 {
    fn from(val: Opcode) -> Self {
        return val.0;
    }
}

/// Advanced: Used in EventBuilder::raw_add_meta to indicate the field's intype.
/// An intype indicates the binary encoding of the field (i.e. how to determine
/// the field's size), e.g. intype I32 indicates the field is always 4 bytes,
/// and intype Str8 indicates the field starts with a U16 byte-count.
/// The intype also provides a default format to be used if no outtype is provided,
/// e.g. I32 has a default outtype "Signed", and Hex32 has a default outtype
/// "Hex".
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InType(u8);

impl InType {
    /// Returns an intype with the specified value.
    /// Requires: `value <= 127`.
    pub const fn from_int(value: u8) -> InType {
        assert!(value <= 127, "InType requires value <= 127");
        return InType(value);
    }

    /// Returns the numeric value corresponding to this intype.
    #[inline(always)]
    pub const fn as_int(self) -> u8 {
        return self.0;
    }

    /// TlgInNULL = Invalid type.
    pub const Invalid: InType = InType(0);

    /// TlgInUNICODESTRING = NUL-terminated UTF-16LE string.
    ///
    /// Default outtype: String.
    ///
    /// Other usable outtypes: Xml, Json.
    pub const Sz16: InType = InType(1);

    /// TlgInANSISTRING = NUL-terminated 8-bit string, assumed to be encoded as CP1252.
    ///
    /// Default outtype: String.
    ///
    /// Other usable outtypes: Xml, Json, Utf8.
    pub const Sz8: InType = InType(2);

    /// TlgInINT8 = 8-bit signed integer.
    ///
    /// Default outtype: Signed.
    ///
    /// Other usable outtypes: String (formats as CP1252 char).
    pub const I8: InType = InType(3);

    /// TlgInUINT8 = 8-bit unsigned integer.
    ///
    /// Default outtype: Unsigned.
    ///
    /// Other usable outtypes: Hex, String (formats as CP1252 char), Boolean.
    pub const U8: InType = InType(4);

    /// TlgInINT16 = 16-bit signed integer.
    ///
    /// Default outtype: Signed.
    pub const I16: InType = InType(5);

    /// TlgInUINT16 = 16-bit unsigned integer.
    ///
    /// Default outtype: Unsigned.
    ///
    /// Other usable outtypes: Hex, String (formats as UCS-2 char), Port.
    pub const U16: InType = InType(6);

    /// TlgInINT32 = 32-bit signed integer.
    ///
    /// Default outtype: Signed.
    ///
    /// Other usable outtypes: HResult.
    pub const I32: InType = InType(7);

    /// TlgInUINT32 = 32-bit unsigned integer.
    ///
    /// Default outtype: Unsigned.
    ///
    /// Other usable outtypes: Pid, Tid, IPv4, Win32Error, NtStatus, CodePointer.
    pub const U32: InType = InType(8);

    /// TlgInINT64 = 64-bit signed integer.
    ///
    /// Default outtype: Signed.
    pub const I64: InType = InType(9);

    /// TlgInUINT64 = 64-bit signed integer.
    ///
    /// Default outtype: Unsigned.
    ///
    /// Other usable outtypes: CodePointer.
    pub const U64: InType = InType(10);

    /// TlgInFLOAT = 32-bit float.
    pub const F32: InType = InType(11);

    /// TlgInDOUBLE = 64-bit float.
    pub const F64: InType = InType(12);

    /// TlgInBOOL32 = 32-bit Boolean.
    ///
    /// Default outtype: Boolean.
    pub const Bool32: InType = InType(13);

    /// TlgInBINARY = UINT16 byte-count followed by binary data.
    ///
    /// Default outtype: Hex.
    ///
    /// Other usable outtypes: IPv6, SocketAddress, Pkcs7WithTypeInfo.
    ///
    /// Note: Binary array is not supported. Use BinaryC array.
    pub const Binary: InType = InType(14);

    /// TlgInGUID = 128-bit GUID in Windows (little-endian) byte order.
    pub const Guid: InType = InType(15);

    /// _TlgInPOINTER_unsupported = Not supported. Use HexSize.
    pub const _HexSize_PlatformSpecific: InType = InType(16);

    /// TlgInFILETIME = 64-bit timestamp in Windows FILETIME format.
    ///
    /// Default outtype: DateTime.
    ///
    /// Other usable outtypes: DateTimeCultureInsensitive, DateTimeUtc.
    pub const FileTime: InType = InType(17);

    /// TlgInSYSTEMTIME = 128-bit date/time in Windows SYSTEMTIME format.
    ///
    /// Default outtype: DateTime.
    ///
    /// Other usable outtypes: DateTimeCultureInsensitive, DateTimeUtc.
    pub const SystemTime: InType = InType(18);

    /// TlgInSID = Security ID win Windows SID format.
    ///
    /// Note: Expected size (sid_length) is `8 + 4 * sid.as_bytes()[1]`.
    pub const Sid: InType = InType(19);

    /// TlgInHEXINT32 = 32-bit integer formatted as hex.
    ///
    /// Default outtype: Hex.
    ///
    /// Other usable outtypes: Win32Error, NtStatus, CodePointer.
    pub const Hex32: InType = InType(20);

    /// TlgInHEXINT64 = 64-bit integer formatted as hex.
    ///
    /// Default outtype: Hex.
    ///
    /// Other usable outtypes: CodePointer.
    pub const Hex64: InType = InType(21);

    /// TlgInCOUNTEDSTRING = 16-bit byte count followed by UTF-16LE string.
    ///
    /// Default outtype: String.
    ///
    /// Other usable outtypes: Xml, Json.
    pub const Str16: InType = InType(22);

    /// TlgInCOUNTEDANSISTRING = 16-bit byte count followed by 8-bit string, assumed
    /// to be encoded as CP1252.
    ///
    /// Default outtype: String.
    ///
    /// Other usable outtypes: Xml, Json, Utf8.
    pub const Str8: InType = InType(23);

    /// _TlgInSTRUCT = The struct field contains no data, but the following N fields
    /// will be considered as logically part of the struct field, where
    /// N is a value from 1 to 127 encoded into the OutType slot.
    pub const Struct: InType = InType(24);

    /// TlgInCOUNTEDBINARY = UINT16 byte-count followed by binary data.
    ///
    /// Default outtype: Hex.
    ///
    /// Other usable outtypes: IPv6, SocketAddress, Pkcs7WithTypeInfo.
    ///
    /// This is the same as Binary except:
    /// - New type code. Decoding supported added in Windows 2018 Fall Update.
    /// - Decodes without the synthesized "FieldName.Length" fields.
    /// - Arrays are supported.
    pub const BinaryC: InType = InType(25);

    /// TlgInINTPTR = an alias for either I64 or I32, depending on the running process's
    /// pointer size.
    ///
    /// Default outtype: Signed.
    pub const ISize: InType = if size_of::<usize>() == 8 {
        InType::I64
    } else {
        InType::I32
    };

    /// TlgInUINTPTR = an alias for either U64 or U32, depending on the running process's
    /// pointer size.
    ///
    /// Default outtype: Unsigned.
    ///
    /// Other usable outtypes: CodePointer.
    pub const USize: InType = if size_of::<usize>() == 8 {
        InType::U64
    } else {
        InType::U32
    };

    /// TlgInPOINTER = an alias for either Hex64 or Hex32, depending on the running
    /// process's pointer size.
    ///
    /// Default outtype: Hex.
    ///
    /// Other usable outtypes: CodePointer.
    pub const HexSize: InType = if size_of::<usize>() == 8 {
        InType::Hex64
    } else {
        InType::Hex32
    };

    /// Raw encoding flag: _TlgInCcount indicates that field metadata contains a
    /// const-array-count slot.
    pub const StaticArrayFlag: u8 = 0x20;

    /// Raw encoding flag: TlgInVcount indicates that field data contains a
    /// variable-array-count slot.
    pub const DynamicArrayFlag: u8 = 0x40;

    /// Raw encoding flag: _TlgInCustom indicates that the field uses a custom
    /// serializer.
    pub const CustomFlag: u8 = 0x60;

    /// Raw encoding flag: _TlgInTypeMask is a mask for the intype portion of the encoded
    /// byte.
    pub const TypeMask: u8 = 0x1F;

    /// Raw encoding flag: _TlgInFlagMask is a mask for the flags portion of the encoded
    /// byte.
    pub const FlagMask: u8 = 0x60;
}

impl fmt::Display for InType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return self.0.fmt(f);
    }
}

impl From<u8> for InType {
    fn from(val: u8) -> Self {
        return Self(val);
    }
}

impl From<InType> for u8 {
    fn from(val: InType) -> Self {
        return val.0;
    }
}

/// Data formatting hints that may be used or ignored by decoders.
///
/// Each field of an event has an InType (specifies the field's binary
/// encoding) and an OutType (formatting hint for the decoder).
///
/// If a field has an OutType set and the decoder supports the field's
/// combination of InType + OutType then the decoder will use the OutType as a
/// formatting hint when decoding the field. For example, a field with
/// InType=U8 is normally formatted as decimal, but if the field sets
/// OutType=Hex and the decoder supports U8+Hex then it would be formatted
/// as hexadecimal, or if the field sets OutType=String and the decoder
/// supports U8+String then it would be formatted as a CP1252 CHAR.
///
/// If the OutType is Default or is not supported by the decoder then the field
/// receives a default formatting based on the InType.
///
/// Note: Setting the OutType to a value other than Default will add 1 byte per
/// field per event in the resulting ETL file. Add a non-Default outtype only if
/// the default outtype is not appropriate.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutType(u8);

impl OutType {
    /// Returns an outtype with the specified value.
    /// Requires: `value <= 127`.
    pub const fn from_int(value: u8) -> OutType {
        assert!(value <= 127, "OutType requires value <= 127");
        return OutType(value);
    }

    /// Returns the numeric value corresponding to this outtype.
    #[inline(always)]
    pub const fn as_int(self) -> u8 {
        return self.0;
    }

    /// TlgOutNULL = default formatting will be applied based on the field's InType.
    pub const Default: OutType = OutType(0);
    /// TlgOutNOPRINT = field should be hidden. (Most decoders show it anyway.)
    pub const NoPrint: OutType = OutType(1);
    /// TlgOutSTRING = field should be formatted as a string. Use with I8, U8, or
    /// U16 to log a char.
    pub const String: OutType = OutType(2);
    /// TlgOutBOOLEAN = field should be formatted as a Bool. Use with U8.
    pub const Boolean: OutType = OutType(3);
    /// TlgOutHEX = field should be formatted as hexadecimal. Use with U8, U16.
    pub const Hex: OutType = OutType(4);
    /// TlgOutPID = field should be formatted as a process id. Use with U32.
    pub const Pid: OutType = OutType(5);
    /// TlgOutTID = field should be formatted as a thread id. Use with U32.
    pub const Tid: OutType = OutType(6);
    /// TlgOutPORT = field should be formatted as a big-endian IP port. Use with U16.
    pub const Port: OutType = OutType(7);
    /// TlgOutIPV4 = field should be formatted as an IPv4 address. Use with U32.
    pub const IPv4: OutType = OutType(8);
    /// TlgOutIPV6 = field should be formatted as an IPv6 address. Use with Binary,
    /// BinaryC.
    pub const IPv6: OutType = OutType(9);
    /// TlgOutSOCKETADDRESS = field should be formatted as a sockaddr. Use with Binary or
    /// BinaryC.
    pub const SocketAddress: OutType = OutType(10);
    /// TlgOutXML = field should be formatted as XML. Use with Str16, Str8, Sz16, and Sz8
    /// types. Implies UTF-8 when used with Str8 or Sz8.
    pub const Xml: OutType = OutType(11);
    /// TlgOutJSON = field should be formatted as JSON. Use with Str16, Str8, Sz16, and Sz8
    /// types. Implies UTF-8 when used with Str8 or Sz8.
    pub const Json: OutType = OutType(12);
    /// TlgOutWIN32ERROR = field should be formatted as a Win32 result code. Use with
    /// U32.
    pub const Win32Error: OutType = OutType(13);
    /// TlgOutNTSTATUS = field should be formatted as a Win32 NTSTATUS. Use with U32.
    pub const NtStatus: OutType = OutType(14);
    /// TlgOutHRESULT = field should be formatted as a Win32 HRESULT. Use with I32.
    pub const HResult: OutType = OutType(15);
    /// TlgOutFILETIME = not generally used. Appropriate intypes already imply DateTime.
    pub const DateTime: OutType = OutType(16);
    /// TlgOutSIGNED = not generally used. Appropriate intypes already imply Signed.
    pub const Signed: OutType = OutType(17);
    /// TlgOutUNSIGNED = not generally used. Appropriate intypes already imply Unsigned.
    pub const Unsigned: OutType = OutType(18);
    /// TlgOutDATETIME_CULTURE_INSENSITIVE = Invariant-culture date-time. Use with
    /// FileTime or SystemTime.
    pub const DateTimeCultureInsensitive: OutType = OutType(33);
    /// TlgOutUTF8 = field should be decoded as UTF-8. Use with Str8 or Sz8.
    pub const Utf8: OutType = OutType(35);
    /// TlgOutPKCS7_WITH_TYPE_INFO = field should be decoded as a PKCS7 packet followed
    /// by TLG type info. Use with Binary or BinaryC.
    pub const Pkcs7WithTypeInfo: OutType = OutType(36);
    /// TlgOutCODE_POINTER = field should be formatted as a code pointer. Use with
    /// U32, U64, USize, Hex32, Hex64, HexSize.
    pub const CodePointer: OutType = OutType(37);
    /// TlgOutDATETIME_UTC = field should be decoded assuming UTC timezone. Use with
    /// FileTime or SystemTime.
    pub const DateTimeUtc: OutType = OutType(38);
    /// _TlgOutTypeMask = raw encoding flag: mask for the outtype portion of the encoded
    /// byte.
    pub const TypeMask: u8 = 0x7F;
}

impl fmt::Display for OutType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return self.0.fmt(f);
    }
}

impl From<u8> for OutType {
    fn from(val: u8) -> Self {
        return Self(val);
    }
}

impl From<OutType> for u8 {
    fn from(val: OutType) -> Self {
        return val.0;
    }
}
