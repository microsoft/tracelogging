"""traceloggingdynamic: Generates ETW TraceLogging events

This module provides support for generating TraceLogging-encoded ETW
(Event Tracing for Windows) events directly from Python (no C/C++ code).

This is a middle-layer API:

- Directly exposes ETW concepts rather than wrapping them in Python idioms.
- Makes it possible for the caller to be efficient even when that means the
  caller might need to do more work (e.g. take names as bytes so that the
  caller has the option of avoiding the string encoding for each event).
- Could be used directly by a consumer or could be wrapped in a higher-level
  module to create a more Python-friendly API.

Public:

- Provider class: represents an ETW data source with a name and id. Core
  methods: provider.is_enabled(event_level, event_keyword),
  provider.write(event_builder).
- EventBuilder class: Used to put together the data for an event. Core
  methods: eb.reset(event_name, event_level, event_keyword, [more...]),
  eb.add_FIELDTYPE(field_name, field_value, [out_type, tag]). When the event
  is ready, send it to ETW with provider.write(eb).
- OutType enum: Formatting hints that can be added to a field.
- providerid_from_name function: Hashes a provider name string to generate
  a provider id UUID. Hash is generated using the same algorithm as other ETW
  APIs and tools such as .NET EventSource and WinRT LoggingChannel.

Example:

    my_field_value = True
    
    provider = Provider(b'MyCompany.MyGroup.MyComponent')
    my_event_level = 5      # 5 = Verbose
    my_event_keyword = 0x21 # User-defined bits indicating categories.
    if provider.is_enabled(my_event_level, my_event_keyword):
        eb = EventBuilder()
        eb.reset(b'MyEventName', my_event_level, my_event_keyword)
        eb.add_bool32(b'MyFieldName', my_field_value) # as needed to add fields.
        provider.write(eb)

General notes:

Collect the events using Windows SDK tools like traceview or tracelog.
Decode the events using Windows SDK tools like traceview or tracefmt.
For example, for Provider('MyCompany.MyComponent'):

    tracelog -start MyTrace -f MyTraceFile.etl -guid *MyCompany.MyComponent -level 5 -matchanykw 0xf
    <run your Python program>
    tracelog -stop MyTrace
    tracefmt -o MyTraceData.txt MyTraceFile.etl

ETW events are limited in size (event size = headers + metadata + data).
Windows will drop any event that is larger than 64KB and will drop any event
that is larger than the buffer size of the recording session. To help prevent
unexpected event loss, this module will raise an exception if the metadata
exceeds 32KB or if the data exceeds 64KB. To help track down event loss, this
module will assert if it detects that ETW dropped an event due to size.

Most ETW decoding tools are unable to decode an event with more than 128
fields. This module will raise an exception if you add more than 128 fields to
an event. Note that sequence fields and binary fields each count as 2 fields.
"""

__all__ = \
    'providerid_from_name', \
    'providerid_from_name_bytes_utf8', \
    'OutType', \
    'EventBuilder', \
    'Provider', \

import typing
import ctypes
import struct
import uuid
import hashlib

def providerid_from_name(providername : str) -> uuid.UUID:
    """Generate a provider id UUID from a provider name string.

    All TraceLogging providers must have a provider name (string) and a
    provider id (UUID). Once selected, the name and id form a pair: the name
    should never be used with any other id, and the id should never be used
    with any other name. The easiest way to ensure this is to generate the id
    as a hash of the name. The hashing used here matches the hashing used by
    several other ETW APIs (e.g. .NET EventSource and WinRT LoggingChannel) as
    well as several ETW tools.

    Note that this algorithm is not case-sensitive.
    """
    # Note: This is almost (but not quite) compliant with RFC 4122 UUIDv5.
    # Not fixed here because it needs to match well-established behavior.
    sha1 = hashlib.sha1(usedforsecurity = False)
    sha1.update(b'\x48\x2C\x2D\xB2\xC3\x90\x47\xC8\x87\xF8\x1A\x15\xBF\xC1\x30\xFB')
    sha1.update(providername.upper().encode('utf_16_be'))
    arr = bytearray(sha1.digest()[0:16])
    arr[7] = (arr[7] & 0x0F) | 0x50
    return uuid.UUID(bytes_le = bytes(arr))

def providerid_from_name_bytes_utf8(providername_utf8 : bytes) -> uuid.UUID:
    """Generate a provider id UUID from a utf-8 encoded provider name string.

    All TraceLogging providers must have a provider name (string) and a
    provider id (UUID). Once selected, the name and id form a pair: the name
    should never be used with any other id, and the id should never be used
    with any other name. The easiest way to ensure this is to generate the id
    as a hash of the name. The hashing used here matches the hashing used by
    several other ETW APIs (e.g. .NET EventSource and WinRT LoggingChannel) as
    well as several ETW tools.

    Note that this algorithm is not case-sensitive.
    """
    return providerid_from_name(str(providername_utf8, encoding='utf-8'))

# Data representation types used in TraceLogging encoding.
# Shown here for documentation purposes (implementation mostly uses literals).
class _InType:

    NULL = 0           # Invalid type.
    UNICODESTRING = 1  # NUL-terminated UTF-16LE string.

    ANSISTRING = 2     # NUL-terminated string of 8-bit characters.
    # Encoding unspecified, usually assumed to be cp1252.
    # If OutType is UTF8, XML, or JSON then encoding is utf-8.

    INT8 = 3           # 8-bit signed integer.
    UINT8 = 4          # 8-bit unsigned integer.
    INT16 = 5          # 16-bit signed integer.
    UINT16 = 6         # 16-bit unsigned integer.
    INT32 = 7          # 32-bit signed integer.
    UINT32 = 8         # 32-bit unsigned integer.
    INT64 = 9          # 64-bit signed integer.
    UINT64 = 10        # 64-bit unsigned integer.
    FLOAT = 11         # 32-bit float.
    DOUBLE = 12        # 64-bit float.
    BOOL32 = 13        # 32-bit bool.
    BINARY = 14        # UINT16 size followed by raw binary data.
    GUID = 15          # 16-byte UUID, in bytes_le order.
    FILETIME = 17      # 64-bit unsigned integer, time since 1601 in 100ns units. Usually UTC.
    SYSTEMTIME = 18    # 16-byte Win32 SYSTEMTIME. Timezone unspecified.
    SID = 19           # Variable-length Win32 SID. Size = 8 + 4 * SubAuthorityCount.
    HEXINT32 = 20      # 32-bit unsigned integer formatted as hex. Shortcut for UINT32+OutType=HEX.
    HEXINT64 = 21      # 64-bit unsigned integer formatted as hex. Shortcut for UINT64+OutType=HEX.
    COUNTEDSTRING = 22 # UINT16 size followed by UTF-16LE string. (Size in bytes, not chars.)

    COUNTEDANSISTRING = 23 # UINT16 size followed by 8-bit character string.
    # Encoding unspecified, usually assumed to be cp1252.
    # If OutType is UTF8, XML, or JSON then encoding is utf-8.

    _STRUCT = 24 # Indicates that the next N logical fields should be considered a struct.
    # The OutType field is used to record the value of N. N must be 1..127.
    # Each struct and its children count as 1 logical field. Each array and its
    # elements count as 1 logical field.

    COUNTEDBINARY = 25 # UINT16 size followed by raw binary data.
    # This is rarely used. This InType is a recent addition (added in Windows 10 2018 Fall
    # Update, so some decoders don't support it yet), and it is mostly equivalent to BINARY.
    # The difference is that COUNTEDBINARY includes the data size as part of the field,
    # while BINARY uses a nominally-separate field for tracking the data size (this is
    # hidden by the TraceLogging system, but it is significant when decoding events using
    # the TDH APIs). As a result, while it is not possible to have an array of BINARY, it is
    # possible to have an array of COUNTEDBINARY, and you won't get the synthesized
    # "fieldname.Length" fields that you get with BINARY fields.

    INTPTR = INT64 if ctypes.sizeof(ctypes.c_void_p) == 8 else INT32
    UINTPTR = UINT64 if ctypes.sizeof(ctypes.c_void_p) == 8 else UINT32
    POINTER = HEXINT64 if ctypes.sizeof(ctypes.c_void_p) == 8 else HEXINT32
    _CCOUNT = 32 # Indicates that field metadata contains a const-array-count tag.
    _VCOUNT = 64 # Indicates that field data contains variable-array-count tag.
    _CHAIN = 128 # Indicates that field metadata contains a TlgOut tag.
    _TYPE_MASK = 31
    _COUNT_MASK = _VCOUNT | _CCOUNT
    _FLAG_MASK = _CHAIN | _VCOUNT | _CCOUNT

class OutType:
    """Data formatting hints that may be used or ignored by decoders.

    Each field of an event has an InType (specifies the field's binary
    encoding) and an OutType (formatting hint for the decoder).

    If a field has an OutType set and the decoder supports the field's
    combination of InType + OutType then the decoder will use the OutType as a
    formatting hint when decoding the field. For example, a field with
    InType=UINT8 is normally formatted as decimal, but if the field sets
    OutType=HEX and the decoder supports UINT8+HEX then it would be formatted
    as hexadecimal, or if the field sets OutType=STRING and the decoder
    supports UINT8+STRING then it would be formatted as a CHAR.

    If the OutType is NULL or is not supported by the decoder then the field
    receives a default formatting based on the InType.

    Note: Setting the OutType to a value other than NULL will add 1 byte per
    field per event in the resulting ETL file.
    """

    NULL = 0
    """Default formatting will be applied based on the field's InType."""

    NOPRINT = 1
    """Field should be hidden. (Most decoders show it anyway.)"""

    STRING = 2
    """Field should be formatted as a string. Use with int8, uint8, or uint16 to log a char."""

    BOOLEAN = 3
    """Field should be formatted as a Bool. Use with uint8."""

    HEX = 4
    """Field should be formatted as hexadecimal. Use with uint8, uint16."""

    PID = 5
    """Field should be formatted as a process id. Use with uint32."""

    TID = 6
    """Field should be formatted as a thread id. Use with uint32."""

    PORT = 7
    """Field should be formatted as a big-endian IP port. Use with uint16."""

    IPV4 = 8
    """Field should be formatted as an IPv4 address. Use with uint32."""

    IPV6 = 9
    """Field should be formatted as an IPv6 address. Use with binary."""

    SOCKETADDRESS = 10
    """Field should be formatted as a sockaddr. Use with binary."""

    XML = 11
    """Field should be formatted as XML. Use with string types. Implies utf-8 for ansistring."""

    JSON = 12
    """Field should be formatted as JSON. Use with string types. Implies utf-8 for ansistring."""

    WIN32ERROR = 13
    """Field should be formatted as a Win32 result code. Use with uint32."""

    NTSTATUS = 14
    """Field should be formatted as a Win32 NTSTATUS. Use with uint32."""

    HRESULT = 15
    """Field should be formatted as a Win32 HRESULT. Use with int32."""

    FILETIME = 16
    """Not generally used. InType=FILETIME already implies OutType=FILETIME."""

    SIGNED = 17
    """Not generally used. InType=INTnn already implies OutType=SIGNED."""

    UNSIGNED = 18
    """Not generally used. InType=UINTnn already implies OutType=UNSIGNED."""

    CULTURE_INSENSITIVE_DATETIME = 33
    """Field should be formatted as locale-invariant. Use with filetime/systemtime."""

    UTF8 = 35
    """Field should be decoded as UTF-8. Use with ansistring types."""

    PKCS7_WITH_TYPE_INFO = 36
    """Field should be decoded as a PKCS7 packet followed by TLG type info. Use with binary."""

    CODE_POINTER = 37
    """Field should be formatted as a code pointer. Use with uint[32|64|ptr], hexint[32|64|ptr]."""

    DATETIME_UTC = 38
    """Field should be decoded assuming UTC timezone. Use with filetime or systemtime."""

    _CHAIN = 128
    _TYPE_MASK = 127

class EventBuilder:
    """Helper for constructing an event. Write the event with Provider.write.

    Use EventBuilder to construct an event that will be written to ETW by a
    Provider.

    Basic usage:

        provider = Provider(b'MyCompany.MyGroup.MyComponent')
        my_event_level = 5      # 5 = Verbose
        my_event_keyword = 0x21 # User-defined bits indicating categories.
        if provider.is_enabled(my_event_level, my_event_keyword):
            eb = EventBuilder()
            eb.reset(b'MyEventName', my_event_level, my_event_keyword)
            eb.add_bool32(b'MyFieldName', my_field_value) # as needed to add fields.
            provider.write(eb)

    An EventBuilder instance can be reused for multiple events by calling
    reset() each time you begin building a new event. This is slightly faster
    than creating a new EventBuilder for each event.

    The add_*** methods are not individually documented, but they all work as
    follows:

    - Each add_*** method adds one field to the event.
    - The add_*** method will raise a ValueError if more than 128 fields are
      added, if metadata grows beyond 32KB, or if data grows beyond 64KB. Once
      an error is hit, subsequent add and write operations will fail until you
      call reset().
    - name_utf8 is a bytes that contains the utf-8 encoded name for the field.
      The name must contain no 0x00 bytes and must be correctly utf-8 encoded.
      The name should be short, meaningful, and unique.
    - val is the value for the field.
    - out_type is an optional formatting hint that may be used by the event
      decoder. Not all hints are supported for all field types.
    - tag is a 28-bit value to be associated with the field. The value can be
      used for any purpose. If not needed, set it to 0.
    """

    # Instance field types
    name_utf8 : bytes
    __field_count : int
    __event_desc : ctypes.c_byte * 16 # EVENT_DESCRIPTOR
    _meta : ctypes.c_byte * 128 # TraceLogging-encoded event name, field names, field types.
    _meta_pos : int
    _data : ctypes.c_byte * 512 # TraceLogging-encoded field values.
    _data_pos : int

    # Static fields
    __FIELD_COUNT_ERR : int = 129 # MAX_EVENT_DATA_DESCRIPTORS + 1
    __struct_event_desc : struct.Struct = struct.Struct(b'=HBBBBHQ') # Id, Ver, Ch, Lvl, Op, Task, Kw
    __struct_traits_tag1 : struct.Struct = struct.Struct(b'=xxB') # UINT16 size, UINT8 Tag
    __struct_traits_tag2 : struct.Struct = struct.Struct(b'=xxH') # UINT16 size, UINT16 Tag
    __struct_traits_tag4 : struct.Struct = struct.Struct(b'=xxI') # UINT16 size, UINT32 Tag
    __struct_i8 : struct.Struct = struct.Struct(b'=b')
    __struct_u8 : struct.Struct = struct.Struct(b'=B')
    __struct_i16 : struct.Struct = struct.Struct(b'=h')
    __struct_u16 : struct.Struct = struct.Struct(b'=H')
    __struct_i32 : struct.Struct = struct.Struct(b'=i')
    __struct_u32 : struct.Struct = struct.Struct(b'=I')
    __struct_i64 : struct.Struct = struct.Struct(b'=q')
    __struct_u64 : struct.Struct = struct.Struct(b'=Q')
    __struct_iptr : struct.Struct = __struct_i64 if ctypes.sizeof(ctypes.c_void_p) == 8 else __struct_i32
    __struct_uptr : struct.Struct = __struct_u64 if ctypes.sizeof(ctypes.c_void_p) == 8 else __struct_u32
    __struct_f32 : struct.Struct = struct.Struct(b'=f')
    __struct_f64 : struct.Struct = struct.Struct(b'=d')

    def __str__(self):

        name = str(self.name_utf8, 'utf-8')
        return f'EventBuilder "{name}"'

    def __init__(self,
        initial_meta_len : int = 128,
        initial_data_len : int = 512,
        ):
        """Creates a new EventBuilder and pre-allocates buffers.

        - initial_meta_len is the byte size to pre-allocate for metadata.
          Minimum is 64. Default is 128. Maximum is 32KB.
        - initial_data_len is the byte size to pre-allocate for data.
          Minimum is 64. Default is 512. Maximum is 64KB.

        The buffers will automatically grow if needed, but event creation may
        be more efficient if appropriate initial buffer sizes are used.
        """

        self.name_utf8 = b''
        self.__field_count = EventBuilder.__FIELD_COUNT_ERR
        self.__event_desc = (ctypes.c_byte * 16)()

        initial = int(initial_meta_len)
        if initial < 64:
            initial = 64
        elif initial > 32768:
            initial = 32768
        self._meta = (ctypes.c_byte * initial)()
        self._meta_pos = 0

        initial = int(initial_data_len)
        if initial < 64:
            initial = 64
        elif initial > 65536:
            initial = 65536
        self._data = (ctypes.c_byte * initial)()
        self._data_pos = 0

    def reset(self,
        name_utf8 : bytes,
        level : int = 5,
        keyword : int = 1,
        opcode : int = 0,
        id : int = 0,
        version : int = 0,
        tag : int = 0,
        task : int = 0,
        channel : int = 11
        ) -> None:
        """Clears any previous state and starts a new event.

        - name_utf8 is the utf-8 encoded name for the event. The name must
          contain no 0x00 bytes and must be correctly utf-8 encoded. The name
          should be short, meaningful, and unique within the provider.
        - level: uint8, default is 5 = verbose.
          1 = fatal, 2 = error, 3 = warning, 4 = info, 5 = verbose.
        - keyword: uint64 flags, default is 0x01 = general category.
          Each bit in the flags is a category that the event is in. All events
          should have at least 1 keyword bit set.
        - opcode: uint8, default is 0 = WINEVENT_OPCODE_INFO.
          Other common values are 1 = WINEVENT_OPCODE_START (indicates that
          this event is starting a new activity), 2 = WINEVENT_OPCODE_STOP
          (indicates that this event is ending a completed activity).
        - tag is a 28-bit value to be associated with the event. The value can
          be used for any purpose. If not needed, set it to 0.
        - id: uint16, default is 0 indicating no assigned stable event id.
        - version: uint8, default is 0. Relevant only if id != 0.
          Stable event version. Increment if an event's fields are changed.
        - task: uint16, default is 0 = none.
          User-defined value for grouping related events.
        - channel: uint8, default is 11 = WINEVENT_CHANNEL_TRACELOGGING.
          Should almost never be anything other than 11.

        Every event should have a non-zero level from 1 (fatal error) to 5
        (verbose). If level is 0, the event cannot be filtered by level.

        Every event should have a non-zero keyword. A keyword is an unsigned
        64-bit integer with each bit indicating a particular category of
        event. An event can be in multiple categories by setting multiple bits
        in the keyword. The top 16 bits are defined by Microsoft and should
        usually be 0. The low 48 bits are defined by the provider owner, e.g.
        the owner of provider "MyCompany.MyGroup.MyComponent" might define bit
        0x02 to mean "networking category". Definitions are scoped to the
        provider, i.e. the "MyCompany.MyGroup.OtherComponent.OtherGroup"
        provider is free to use 0x02 for "User Interface category". If the
        keyword is 0, the event cannot be filtered by keyword.

        For a given event name, the level, keyword, opcode, tag, id, task,
        channel, field names, and field types should always be the same. For
        example, don't dynamically change the level without also changing the
        event name.
        """

        if tag & 0x0FE00000 == tag:
            tag_len = 1
        elif tag & 0x0FFFC000 == tag:
            tag_len = 2
        elif tag & 0x0FFFFFFF == tag:
            tag_len = 4
        else:
            raise ValueError('Invalid tag value. Tag must fit in 28 bits or less.')

        assert 0 > name_utf8.find(b'\0'), 'Event name must not contain 0x00 bytes.'

        EventBuilder.__struct_event_desc.pack_into(self.__event_desc, 0,
            id, version, channel, level, opcode, task, keyword)

        name_len = len(name_utf8)
        traits_len = 2 \
            + tag_len \
            + name_len + 1

        if len(self._meta) < traits_len: self.__grow_meta(traits_len)

        meta = self._meta
        meta_pos = 0
        if tag_len == 1:
            EventBuilder.__struct_traits_tag1.pack_into(meta, meta_pos,
                (tag & 0x0FE00000) >> 21)
            meta_pos += 3
        elif tag_len == 2:
            EventBuilder.__struct_traits_tag2.pack_into(meta, meta_pos,
                0x80  \
                | ((tag & 0xfe00000) >> 21) \
                | ((tag & 0x01fc000) >> 6))
            meta_pos += 4
        else:
            assert(tag_len == 4)
            EventBuilder.__struct_traits_tag4.pack_into(meta, meta_pos,
                0x808080 \
                | ((tag & 0xfe00000) >> 21) \
                | ((tag & 0x01fc000) >> 6)  \
                | ((tag & 0x0003f80) << 9)  \
                | ((tag & 0x000007f) << 24))
            meta_pos += 6
        meta[meta_pos:meta_pos + name_len] = name_utf8
        meta_pos += name_len
        meta[meta_pos] = 0
        meta_pos += 1

        self._meta_pos = meta_pos
        self._data_pos = 0
        self.__field_count = 0
        self.name_utf8 = name_utf8

    def check_state(self) -> None:
        """Raises a ValueError if the EventBuilder is in an error state.

        Error state occurs if an add_*** method is called that causes the
        number of fields to exceed 128, the size of metadata to exceed 32KB,
        or the size of data to exceed 64KB.

        Error state can be cleared by calling the reset() method.
        """

        if self.__field_count > 128:
            raise ValueError('EventBuilder is in an error state. Call builder.reset().')

    def add_zstr16(self, name_utf8 : bytes, val_str : str, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a nul-terminated string of utf-16le characters.

        Note: add_str (counted string encoding) is slightly more efficient
        than add_zstr (nul-terminated string encoding). Use add_zstr only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta(name_utf8, 1, out_type, tag) # 1 = InType.UNICODESTRING
        self.__add_data_zstr16(val_str)

    def add_zstr16_seq(self, name_utf8 : bytes, vals_str : typing.Sequence[str], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of nul-terminated strings of utf-16le characters.

        Note: add_str_seq (counted string encoding) is slightly more efficient
        than add_zstr16_seq (nul-terminated string encoding). Use add_zstr16_seq only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta_seq(name_utf8, 1, out_type, tag) # 1 = InType.UNICODESTRING
        self.__add_data_packed(len(vals_str), EventBuilder.__struct_u16)
        for val in vals_str:
            self.__add_data_zstr16(val)

    def add_zstr16_bytes_utf16le(self, name_utf8 : bytes, val_bytes_utf16le : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a nul-terminated string of utf-16le characters.

        Note: add_str (counted string encoding) is slightly more efficient
        than add_zstr (nul-terminated string encoding). Use add_zstr only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta(name_utf8, 1, out_type, tag) # 1 = InType.UNICODESTRING
        self.__add_data_zstr16_bytes_utf16le(val_bytes_utf16le)

    def add_zstr16_bytes_utf16le_seq(self, name_utf8 : bytes, vals_bytes_utf16le : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of nul-terminated strings of utf-16le characters.

        Note: add_str_seq (counted string encoding) is slightly more efficient
        than add_zstr_seq (nul-terminated string encoding). Use add_zstr_seq only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta_seq(name_utf8, 1, out_type, tag) # 1 = InType.UNICODESTRING
        self.__add_data_packed(len(vals_bytes_utf16le), EventBuilder.__struct_u16)
        for val in vals_bytes_utf16le:
            self.__add_data_zstr16_bytes_utf16le(val)

    def add_zstr8(self, name_utf8 : bytes, val_str : str, out_type : int = OutType.UTF8, tag : int = 0) -> None:
        """Adds a field with a nul-terminated string of utf-8 characters.

        val_str will be encoded as utf-8. out_type defaults to UTF8.
        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),

        Note: add_str (counted string encoding) is slightly more efficient
        than add_zstr (nul-terminated string encoding). Use add_zstr only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta(name_utf8, 2, out_type, tag) # 2 = InType.ANSISTRING
        self.__add_data_zstr8(val_str)

    def add_zstr8_seq(self, name_utf8 : bytes, vals_str : typing.Sequence[str], out_type : int = OutType.UTF8, tag : int = 0) -> None:
        """Adds a field with a sequence of nul-terminated strings of utf-8 characters.

        vals_str will be encoded as utf-8. out_type defaults to UTF8.
        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),

        Note: add_str_seq (counted string encoding) is slightly more efficient
        than add_zstr_seq (nul-terminated string encoding). Use add_zstr_seq only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta_seq(name_utf8, 2, out_type, tag) # 2 = InType.ANSISTRING
        self.__add_data_packed(len(vals_str), EventBuilder.__struct_u16)
        for val in vals_str:
            self.__add_data_zstr8(val)

    def add_zstr8_bytes(self, name_utf8 : bytes, val_bytes : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a nul-terminated string of 8-bit characters.

        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),

        Note: add_str (counted string encoding) is slightly more efficient
        than add_zstr (nul-terminated string encoding). Use add_zstr only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta(name_utf8, 2, out_type, tag) # 2 = InType.ANSISTRING
        self.__add_data_zstr8_bytes(val_bytes)

    def add_zstr8_bytes_seq(self, name_utf8 : bytes, vals_bytes : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of nul-terminated strings of 8-bit characters.

        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),

        Note: add_str_seq (counted string encoding) is slightly more efficient
        than add_zstr_seq (nul-terminated string encoding). Use add_zstr_seq only if
        your ETW consumer requires nul-terminated strings.
        """
        self.__add_meta_seq(name_utf8, 2, out_type, tag) # 2 = InType.ANSISTRING
        self.__add_data_packed(len(vals_bytes), EventBuilder.__struct_u16)
        for val in vals_bytes:
            self.__add_data_zstr8_bytes(val)

    def add_int8(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 3, out_type, tag) # 3 = InType.INT8
        self.__add_data_packed(val_int, EventBuilder.__struct_i8)

    def add_int8_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 3, out_type, tag) # 3 = InType.INT8
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i8)

    def add_uint8(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 4, out_type, tag) # 4 = InType.UINT8
        self.__add_data_packed(val_int, EventBuilder.__struct_i8)

    def add_uint8_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 4, out_type, tag) # 4 = InType.UINT8
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i8)

    def add_int16(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 5, out_type, tag) # 5 = InType.INT16
        self.__add_data_packed(val_int, EventBuilder.__struct_i16)

    def add_int16_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 5, out_type, tag) # 5 = InType.INT16
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i16)

    def add_uint16(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 6, out_type, tag) # 6 = InType.UINT16
        self.__add_data_packed(val_int, EventBuilder.__struct_u16)

    def add_uint16_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 6, out_type, tag) # 6 = InType.UINT16
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_u16)

    def add_int32(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 7, out_type, tag) # 7 = InType.INT32
        self.__add_data_packed(val_int, EventBuilder.__struct_i32)

    def add_int32_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 7, out_type, tag) # 7 = InType.INT32
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i32)

    def add_uint32(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 8, out_type, tag) # 8 = InType.UINT32
        self.__add_data_packed(val_int, EventBuilder.__struct_u32)

    def add_uint32_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 8, out_type, tag) # 8 = InType.UINT32
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_u32)

    def add_int64(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 9, out_type, tag) # 9 = InType.INT64
        self.__add_data_packed(val_int, EventBuilder.__struct_i64)

    def add_int64_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 9, out_type, tag) # 9 = InType.INT64
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i64)

    def add_uint64(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 10, out_type, tag) # 10 = InType.UINT64
        self.__add_data_packed(val_int, EventBuilder.__struct_u64)

    def add_uint64_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 10, out_type, tag) # 10 = InType.UINT64
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_u64)

    def add_intptr(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a pointer-sized (32 or 64-bit) signed integer."""
        self.__add_meta(name_utf8, _InType.INTPTR, out_type, tag)
        self.__add_data_packed(val_int, EventBuilder.__struct_iptr)

    def add_intptr_seq(self, name_utf8 : bytes, vals_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a pointer-sized (32 or 64-bit) signed integer."""
        self.__add_meta_seq(name_utf8, _InType.INTPTR, out_type, tag)
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_iptr)

    def add_uintptr(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a pointer-sized (32 or 64-bit) unsigned integer."""
        self.__add_meta(name_utf8, _InType.UINTPTR, out_type, tag)
        self.__add_data_packed(val_int, EventBuilder.__struct_uptr)

    def add_uintptr_seq(self, name_utf8 : bytes, vals_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a pointer-sized (32 or 64-bit) unsigned integer."""
        self.__add_meta_seq(name_utf8, _InType.UINTPTR, out_type, tag)
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_uptr)

    def add_float32(self, name_utf8 : bytes, val_float : float, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 11, out_type, tag) # 11 = InType.FLOAT
        self.__add_data_packed(val_float, EventBuilder.__struct_f32)

    def add_float32_seq(self, name_utf8 : bytes, vals_float : typing.Sequence[float], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 11, out_type, tag) # 11 = InType.FLOAT
        self.__add_data_packed_seq(vals_float, EventBuilder.__struct_f32)

    def add_float64(self, name_utf8 : bytes, val_float : float, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 12, out_type, tag) # 12 = InType.DOUBLE
        self.__add_data_packed(val_float, EventBuilder.__struct_f64)

    def add_float64_seq(self, name_utf8 : bytes, vals_float : typing.Sequence[float], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 12, out_type, tag) # 12 = InType.DOUBLE
        self.__add_data_packed_seq(vals_float, EventBuilder.__struct_f64)

    def add_bool32(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 13, out_type, tag) # 13 = InType.BOOL32
        self.__add_data_packed(val_int, EventBuilder.__struct_i32)

    def add_bool32_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 13, out_type, tag) # 13 = InType.BOOL32
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i32)

    def add_binary_bytes(self, name_utf8 : bytes, val_bytes : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__field_count += 1
        self.__add_meta(name_utf8, 14, out_type, tag) # 14 = InType.BINARY
        self.__add_data_counted(val_bytes)

    # add_binary_bytes_seq not provided: TDH cannot decode it. Use add_cbinary_bytes_seq instead.

    def add_guid(self, name_utf8 : bytes, val_uuid : uuid.UUID, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 15, out_type, tag) # 15 = InType.GUID
        self.__add_data_bytes(val_uuid.bytes_le)

    def add_guid_seq(self, name_utf8 : bytes, vals_uuid : typing.Sequence[uuid.UUID], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 15, out_type, tag) # 15 = InType.GUID
        self.__add_data_packed(len(vals_uuid), EventBuilder.__struct_u16)
        for val in vals_uuid:
            self.__add_data_bytes(val.bytes_le)

    def add_guid_bytes_le(self, name_utf8 : bytes, val_bytes_le: bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """val_bytes_le must be 16 bytes, e.g. from UUID.bytes_le."""
        self.__add_meta(name_utf8, 15, out_type, tag) # 15 = InType.GUID
        if len(val_bytes_le) != 16: self.__raise_ValueError('GUID val_bytes_le must be 16 bytes')
        self.__add_data_bytes(val_bytes_le)

    def add_guid_bytes_le_seq(self, name_utf8 : bytes, vals_bytes_le: typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Each value in vals_bytes_le must be bytes of length 16, e.g. from UUID.bytes_le."""
        self.__add_meta_seq(name_utf8, 15, out_type, tag) # 15 = InType.GUID
        self.__add_data_packed(len(vals_bytes_le), EventBuilder.__struct_u16)
        for val in vals_bytes_le:
            if len(val) != 16: self.__raise_ValueError('GUID val_bytes_le must be 16 bytes')
            self.__add_data_bytes(val)

    def add_filetime_int64(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """val is a positive int64 with time since 1601, in 100ns units (Win32 FILETIME structure)."""
        self.__add_meta(name_utf8, 17, out_type, tag) # 17 = InType.FILETIME
        self.__add_data_packed(val_int, EventBuilder.__struct_i64)

    def add_filetime_int64_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Each val is a positive int64 with time since 1601, in 100ns units (Win32 FILETIME structure)."""
        self.__add_meta_seq(name_utf8, 17, out_type, tag) # 17 = InType.FILETIME
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_i64)

    def add_systemtime_bytes(self, name_utf8 : bytes, val_bytes : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """val_bytes must be 16 bytes (Win32 SYSTEMTIME structure)."""
        self.__add_meta(name_utf8, 18, out_type, tag) # 18 = InType.SYSTEMTIME
        if len(val_bytes) != 16: self.__raise_ValueError('SYSTEMTIME val_bytes must be 16 bytes')
        self.__add_data_bytes(val_bytes)

    def add_systemtime_bytes_seq(self, name_utf8 : bytes, vals_bytes : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Each value in vals_bytes must be bytes of length 16 (Win32 SYSTEMTIME structure)."""
        self.__add_meta_seq(name_utf8, 18, out_type, tag) # 18 = InType.SYSTEMTIME
        self.__add_data_packed(len(vals_bytes), EventBuilder.__struct_u16)
        for val in vals_bytes:
            if len(val) != 16: self.__raise_ValueError('SYSTEMTIME val_bytes must be 16 bytes')
            self.__add_data_bytes(val)

    def add_sid_bytes(self, name_utf8 : bytes, val_bytes : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """val_bytes must be (8 + 4 * SubAuthorityCount) bytes (Win32 SID structure)."""
        self.__add_meta(name_utf8, 19, out_type, tag) # 19 = InType.SID
        if len(val_bytes) != 8 + val_bytes[1] * 4: self.__raise_ValueError('SID val_bytes must be (8 + 4 * SubAuthorityCount) bytes.')
        self.__add_data_bytes(val_bytes)

    def add_sid_bytes_seq(self, name_utf8 : bytes, vals_bytes : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Each value in vals_bytes must be (8 + 4 * SubAuthorityCount) bytes (Win32 SID structure)."""
        self.__add_meta_seq(name_utf8, 19, out_type, tag) # 19 = InType.SID
        self.__add_data_packed(len(vals_bytes), EventBuilder.__struct_u16)
        for val in vals_bytes:
            if len(val) != 8 + val[1] * 4: self.__raise_ValueError('SID val_bytes must be (8 + 4 * SubAuthorityCount) bytes.')
            self.__add_data_bytes(val)

    def add_hexint32(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 20, out_type, tag) # 20 = InType.HEXINT32
        self.__add_data_packed(val_int, EventBuilder.__struct_u32)

    def add_hexint32_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 20, out_type, tag) # 20 = InType.HEXINT32
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_u32)

    def add_hexint64(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta(name_utf8, 21, out_type, tag) # 21 = InType.HEXINT64
        self.__add_data_packed(val_int, EventBuilder.__struct_u64)

    def add_hexint64_seq(self, name_utf8 : bytes, vals_int : typing.Sequence[int], out_type : int = OutType.NULL, tag : int = 0) -> None:
        self.__add_meta_seq(name_utf8, 21, out_type, tag) # 21 = InType.HEXINT64
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_u64)

    def add_hexintptr(self, name_utf8 : bytes, val_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a pointer-sized (32 or 64-bit) unsigned hexadecimal integer."""
        self.__add_meta(name_utf8, _InType.POINTER, out_type, tag)
        self.__add_data_packed(val_int, EventBuilder.__struct_uptr)

    def add_hexintptr_seq(self, name_utf8 : bytes, vals_int : int, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a pointer-sized (32 or 64-bit) unsigned hexadecimal integer."""
        self.__add_meta_seq(name_utf8, _InType.POINTER, out_type, tag)
        self.__add_data_packed_seq(vals_int, EventBuilder.__struct_uptr)

    def add_str16(self, name_utf8 : bytes, val_str : str, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a counted string of utf-16le characters."""
        self.__add_meta(name_utf8, 22, out_type, tag) # 22 = InType.COUNTEDSTRING
        val_bytes_utf16le = val_str.encode(encoding='utf_16_le')
        assert 0 == (len(val_bytes_utf16le) & 1)
        self.__add_data_counted(val_bytes_utf16le)

    def add_str16_seq(self, name_utf8 : bytes, vals_str : typing.Sequence[str], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of counted strings of utf-16le characters."""
        self.__add_meta_seq(name_utf8, 22, out_type, tag) # 22 = InType.COUNTEDSTRING
        self.__add_data_packed(len(vals_str), EventBuilder.__struct_u16)
        for val in vals_str:
            val_bytes_utf16le = val.encode(encoding='utf_16_le')
            assert 0 == (len(val_bytes_utf16le) & 1)
            self.__add_data_counted(val_bytes_utf16le)

    def add_str16_bytes_utf16le(self, name_utf8 : bytes, val_bytes_utf16le : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a counted string of utf-16le characters."""
        self.__add_meta(name_utf8, 22, out_type, tag) # 22 = InType.COUNTEDSTRING
        assert 0 == (len(val_bytes_utf16le) & 1), 'len(val) must be even (expected utf-16le bytes)'
        self.__add_data_counted(val_bytes_utf16le)

    def add_str16_bytes_utf16le_seq(self, name_utf8 : bytes, vals_bytes_utf16le : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of counted strings of utf-16le characters."""
        self.__add_meta_seq(name_utf8, 22, out_type, tag) # 22 = InType.COUNTEDSTRING
        self.__add_data_packed(len(vals_bytes_utf16le), EventBuilder.__struct_u16)
        for val in vals_bytes_utf16le:
            assert 0 == (len(val) & 1), 'len(val) must be even (expected utf-16le bytes)'
            self.__add_data_counted(val)

    def add_str8(self, name_utf8 : bytes, val_str : str, out_type : int = OutType.UTF8, tag : int = 0) -> None:
        """Adds a field with a counted string of utf-8 characters.

        val_str will be encoded as utf-8. out_type defaults to UTF8.
        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),
        """
        self.__add_meta(name_utf8, 23, out_type, tag) # 23 = InType.COUNTEDANSISTRING
        self.__add_data_counted(val_str.encode())

    def add_str8_seq(self, name_utf8 : bytes, vals_str : typing.Sequence[str], out_type : int = OutType.UTF8, tag : int = 0) -> None:
        """Adds a field with a sequence of counted strings of utf-8 characters.

        val_str will be encoded as utf-8. out_type defaults to UTF8.
        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),
        """
        self.__add_meta_seq(name_utf8, 23, out_type, tag) # 23 = InType.COUNTEDANSISTRING
        self.__add_data_packed(len(vals_str), EventBuilder.__struct_u16)
        for val in vals_str:
            self.__add_data_counted(val.encode())

    def add_str8_bytes(self, name_utf8 : bytes, val_bytes : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a counted string of 8-bit characters.

        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),
        """
        self.__add_meta(name_utf8, 23, out_type, tag) # 23 = InType.COUNTEDANSISTRING
        self.__add_data_counted(val_bytes)

    def add_str8_bytes_seq(self, name_utf8 : bytes, vals_bytes : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of counted strings of 8-bit characters.

        Set out_type to UTF8, XML, or JSON for utf-8. If out_type is anything
        else, the encoding is unspecified (most decoders will assume cp1252),
        """
        self.__add_meta_seq(name_utf8, 23, out_type, tag) # 23 = InType.COUNTEDANSISTRING
        self.__add_data_packed(len(vals_bytes), EventBuilder.__struct_u16)
        for val in vals_bytes:
            self.__add_data_counted(val)

    def add_cbinary_bytes(self, name_utf8 : bytes, val_bytes : bytes, out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a counted binary value (decoding may fail before Windows 10 2018 Fall Update).

        Same as add_binary_bytes but uses a newer encoding. Decoding this field
        requires Windows 10 2018 Fall Update or later.
        """
        self.__add_meta(name_utf8, 25, out_type, tag) # 25 = InType.COUNTEDBINARY
        self.__add_data_counted(val_bytes)

    def add_cbinary_bytes_seq(self, name_utf8 : bytes, vals_bytes : typing.Sequence[bytes], out_type : int = OutType.NULL, tag : int = 0) -> None:
        """Adds a field with a sequence of counted binary values (decoding may fail before Windows 10 2018 Fall Update)."""
        self.__add_meta_seq(name_utf8, 25, out_type, tag) # 25 = InType.COUNTEDBINARY
        self.__add_data_packed(len(vals_bytes), EventBuilder.__struct_u16)
        for val in vals_bytes:
            self.__add_data_counted(val)

    def add_struct(self, name_utf8 : bytes, field_count : int, tag : int = 0) -> None:
        """Groups event fields together in a struct with field_count logical fields.

        Creates a struct that will consist of the next field_count
        logical fields, where field_count is in the range 1 to 127.
        Structs can nest, and a struct counts as 1 logical field.
        """
        assert(1 <= field_count <= 127)
        self.__add_meta(name_utf8, 24, field_count, tag) # 24 = InType.STRUCT

    # add_struct_seq not provided.

    @staticmethod
    def __left(val, slice_len):
        assert slice_len <= len(val)
        return val if len(val) == slice_len else val[:slice_len]

    def _get_event_desc(self) -> ctypes.c_byte * 16:

        if self.__field_count > 128: self.check_state()
        return self.__event_desc

    def __add_meta(self, name_utf8 : bytes, in_type : int, out_type : int, tag : int) -> None:
        """Adds a field entry to the event's metadata."""

        assert 0 > name_utf8.find(b'\0'), 'Field name must not contain 0x00 bytes.'

        if self.__field_count >= 128:
            self.check_state() # Throw if already in an error state.
            self.__raise_ValueError('Too many fields (limit 128)')
        self.__field_count += 1

        name_len = len(name_utf8)

        if tag != 0:
            val_len = name_len + 7 # 7 = nul + intype + outtype + tag
            in_val = (in_type & 127) | 128
        elif out_type != 0:
            val_len = name_len + 3 # 3 = nul + intype + outtype
            in_val = (in_type & 127) | 128
        else:
            val_len = name_len + 2 # 2 = nul + intype
            in_val = in_type & 127

        meta = self._meta
        meta_pos = self._meta_pos
        new_pos = meta_pos + val_len

        if len(meta) < new_pos:
            self.__grow_meta(new_pos)
            meta = self._meta

        meta[meta_pos:meta_pos + name_len] = name_utf8
        meta_pos += name_len
        meta[meta_pos] = 0 # nul
        meta_pos += 1
        meta[meta_pos] = in_val # intype
        meta_pos += 1

        if tag != 0:
            meta[meta_pos] = (out_type & 127) | 128
            meta_pos += 1
            EventBuilder.__struct_u32.pack_into(meta, meta_pos, 0x808080 \
                | ((tag & 0x7f) << 24) \
                | ((tag & 0x3f80) << 9) \
                | ((tag & 0x1fc000) >> 6) \
                | ((tag & 0xfe00000) >> 21))
            meta_pos += 4
        elif out_type != 0:
            meta[meta_pos] = out_type & 127
            meta_pos += 1

        self._meta_pos = meta_pos

    def __add_meta_seq(self, name_utf8 : bytes, in_type : int, out_type : int, tag : int) -> None:
        self.__field_count += 1
        self.__add_meta(name_utf8, in_type | 64, out_type, tag)

    def __add_data_zstr16(self, val_str : str) -> None:
        nul_pos = val_str.find('\0')
        if nul_pos < 0:
            val_bytes_utf16le = val_str.encode(encoding='utf_16_le')
            assert 0 == (len(val_bytes_utf16le) & 1)
            self.__add_data_bytes_z(val_bytes_utf16le, EventBuilder.__struct_u16)
        else:
            val_bytes_utf16le = EventBuilder.__left(val_str, nul_pos + 1).encode(encoding='utf_16_le')
            assert 0 == (len(val_bytes_utf16le) & 1)
            self.__add_data_bytes(val_bytes_utf16le)

    def __add_data_zstr16_bytes_utf16le(self, val_bytes_utf16le : bytes) -> None:
        assert 0 == (len(val_bytes_utf16le) & 1), 'len(val) must be even (expected utf-16le bytes)'
        nul_pos = 0
        while 1:
            nul_pos = val_bytes_utf16le.find(b'\0\0', nul_pos)
            if nul_pos < 0:
                self.__add_data_bytes_z(val_bytes_utf16le, EventBuilder.__struct_u16)
                break
            elif (nul_pos & 1) == 0: # Only valid nul-termination if pos is even.
                self.__add_data_bytes(EventBuilder.__left(val_bytes_utf16le, nul_pos + 2))
                break
            nul_pos += 1

    def __add_data_zstr8(self, val_str : str) -> None:
        nul_pos = val_str.find('\0')
        if nul_pos < 0:
            self.__add_data_bytes_z(val_str.encode(), EventBuilder.__struct_u8)
        else:
            self.__add_data_bytes(EventBuilder.__left(val_str, nul_pos + 1).encode())

    def __add_data_zstr8_bytes(self, val_bytes : bytes) -> None:
        nul_pos = val_bytes.find(b'\0')
        if nul_pos < 0:
            self.__add_data_bytes_z(val_bytes, EventBuilder.__struct_u8)
        else:
            self.__add_data_bytes(EventBuilder.__left(val_bytes, nul_pos + 1))

    def __add_data_packed(self, val : int, packer : struct.Struct) -> None:
        """Adds a Struct-packed value to the event's data."""

        data = self._data
        pos = self._data_pos
        new_pos = pos + packer.size

        if len(data) < new_pos:
            self.__grow_data(new_pos)
            data = self._data

        packer.pack_into(data, pos, val)
        self._data_pos = new_pos

    def __add_data_packed_seq(self, vals : typing.Sequence[int], packer : struct.Struct) -> None:
        """Adds a Struct-packed sequence to the event's data."""

        data = self._data
        pos = self._data_pos
        packer_size = packer.size
        vals_len = len(vals)
        new_pos = pos + 2 + packer_size * vals_len

        if len(data) < new_pos:
            self.__grow_data(new_pos)
            data = self._data

        EventBuilder.__struct_u16.pack_into(data, pos, vals_len)
        pos += 2

        for val in vals:
            packer.pack_into(data, pos, val)
            pos += packer_size

        assert pos == new_pos
        self._data_pos = new_pos

    def __add_data_bytes(self, val : bytes) -> None:
        """Adds raw bytes to the event's data."""

        data = self._data
        old_pos = self._data_pos
        new_pos = old_pos + len(val)

        if len(data) < new_pos:
            self.__grow_data(new_pos)
            data = self._data

        data[old_pos:new_pos] = val
        self._data_pos = new_pos

    def __add_data_bytes_z(self, val : bytes, nul_packer : struct.Struct) -> None:
        """Adds raw bytes + NUL to the event's data."""

        data = self._data
        old_pos = self._data_pos
        nul_pos = old_pos + len(val)
        new_pos = nul_pos + nul_packer.size

        if len(data) < new_pos:
            self.__grow_data(new_pos)
            data = self._data

        data[old_pos:nul_pos] = val
        nul_packer.pack_into(data, nul_pos, 0)
        self._data_pos = new_pos

    def __add_data_counted(self, val : bytes) -> None:
        """Adds size16 + raw bytes to the event's data.

        This is compatible with BINARY, COUNTEDSTRING, COUNTEDANSISTRING, and
        COUNTEDBINARY.
        """

        val_len = len(val)

        data = self._data
        data_pos = self._data_pos
        new_pos = data_pos + 2 + val_len

        if len(data) < new_pos:
            self.__grow_data(new_pos)
            data = self._data

        EventBuilder.__struct_u16.pack_into(data, data_pos, val_len)
        data_pos += 2
        data[data_pos:new_pos] = val
        self._data_pos = new_pos

    def __grow_meta(self, needed_len : int) -> None:

        old = self._meta
        old_len = len(old)
        new_len = old_len
        while new_len < needed_len:
            new_len *= 2
        if new_len > 32768: self.__raise_ValueError('Metadata too long')
        new = (ctypes.c_byte * new_len)()
        new[0:old_len] = old
        self._meta = new

    def __grow_data(self, needed_len : int) -> bool:

        old = self._meta
        old_len = len(old)
        new_len = old_len
        while new_len < needed_len:
            new_len *= 2
        if new_len > 65536: self.__raise_ValueError('Data too long')
        new = (ctypes.c_byte * new_len)()
        new[0:old_len] = old
        self._data = new
        return True

    def __raise_ValueError(self, msg : str) -> None:
        self.__field_count = EventBuilder.__FIELD_COUNT_ERR
        raise ValueError(msg)

class Provider:
    """Represents a connection to the ETW system. Used to write events to ETW.

    Use a Provider to write EventBuilder events to ETW. Creating a Provider
    calls the Win32 EventRegister API. The Provider's write method calls
    EventWriteTransfer. The Provider calls EventUnregister when it is deleted.

    Basic usage:

        provider = Provider(b'MyCompany.MyGroup.MyComponent')
        my_event_level = 5      # 5 = Verbose
        my_event_keyword = 0x21 # User-defined bits indicating categories.
        if provider.is_enabled(my_event_level, my_event_keyword):
            eb = EventBuilder()
            eb.reset(b'MyEventName', my_event_level, my_event_keyword)
            eb.add_bool32(b'MyFieldName', my_field_value) # as needed to add fields.
            provider.write(eb)

    Notes:

    Every provider has a name (string) and an id (UUID). The name and id form
    a pair (the name should not be used with any other id, and the id should
    not be used with any other name). The recommended practice is to generate
    the id as a hash of the name, which happens automatically if no id is
    specified when the Provider is constructed. (You can also use the
    providerid_from_name function to do this.)

    By default, the Provider constructor does not raise exceptions, even if
    EventRegister returns an error. This is a good default behavior for
    retail software (most programs should run normally even if there is a
    problem with the diagnostic logging). For debugging, you may want to set
    throw_on_error = True to get an exception if EventRegister fails.

    The Provider.write(event) method calls EventWriteTransfer and returns the
    result. EventWriteTransfer returns 0 if the event was successfully
    written to everybody who is listening to the event, 0 if nobody is
    listening for the event, and a non-zero Win32 error code otherwise.
    EventWriteTransfer can fail for various reasons, some of which should be
    ignored (nothing you can do about them) and some of which might indicate
    bugs in your program or some problem with your ETW configuration. In
    release mode, you should generally ignore this error code, but consider
    checking for errors in debug mode or when diagnosing ETW problems.
    """

    # Instance fields
    __handle: ctypes.c_uint64
    __traits : (ctypes.c_byte * 128)
    name_utf8 : bytes # utf-8 encoded name
    id : uuid.UUID
    group_id : typing.Union[None, uuid.UUID]

    # Static fields
    __null_handle : ctypes.c_uint64 = ctypes.c_uint64()
    __data_desc : ctypes.c_byte * 48 = (ctypes.c_byte * 48)()  # EVENT_DATA_DESCRIPTOR * 3
    __struct_data_desc : struct.Struct = struct.Struct(b'=QII') # Ptr, Size, Reserved
    __struct_u16 : struct.Struct = struct.Struct(b'=H')
    __struct_group : struct.Struct = struct.Struct(b'=HB16s')

    # Prefer the APIset over advapi32.dll. Should be present on Win8 or later.
    try:
        # 0x00000800 = LOAD_LIBRARY_SEARCH_SYSTEM32 (Win8 or later)
        __eventing_dll = ctypes.WinDLL('api-ms-win-eventing-provider-l1-1-0.dll', mode = 0x00000800)
    except:
        # Use advapi32.dll for for Windows 7.
        __eventing_dll = ctypes.WinDLL('advapi32.dll')

    __EventUnregister = __eventing_dll.EventUnregister
    __EventUnregister.restype = ctypes.c_uint32
    __EventUnregister.argtypes = (
        ctypes.c_uint64,
        )

    __EventRegister = __eventing_dll.EventRegister
    __EventRegister.restype = ctypes.c_uint32
    __EventRegister.argtypes = (
        ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint64))

    __EventProviderEnabled = __eventing_dll.EventProviderEnabled
    __EventProviderEnabled.restype = ctypes.c_byte
    __EventProviderEnabled.argtypes = (
        ctypes.c_uint64, ctypes.c_byte, ctypes.c_uint64)

    __EventWriteTransfer = __eventing_dll.EventWriteTransfer
    __EventWriteTransfer.restype = ctypes.c_uint32
    __EventWriteTransfer.argtypes = (
        ctypes.c_uint64, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_void_p)

    # EventSetInformation improves support but we can work without it.
    # The primary issue is that if the system doesn't have EventSetInformation
    # then any event logged with channel != 11 will not decode correctly.
    try:
        __EventSetInformation = __eventing_dll.EventSetInformation
        __EventSetInformation.restype = ctypes.c_uint32
        __EventSetInformation.argtypes = (
            ctypes.c_uint64, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_uint32)
    except:
        @staticmethod
        def __EventSetInformationNoOp(handle, data_type, data, size):
            del handle
            del data_type
            del data
            del size
            return 0
        __EventSetInformation = __EventSetInformationNoOp

    def __str__(self):

        name = str(self.name_utf8, 'utf-8')
        return f'Provider "{name}" {{{self.id}}}'

    def __del__(self):
        self.close()

    def __init__(self,
        name_utf8 : bytes,
        id : typing.Union[None, uuid.UUID] = None,
        group_id : typing.Union[None, uuid.UUID] = None,
        throw_on_error : bool = False,
        ):
        """Creates a new Provider and calls EventRegister.

        - name_utf8 is the utf-8 encoded name for the provider. The name must
          not be empty, must contain no 0x00 bytes and must be correctly utf-8
          encoded. The name should be short, meaningful, and unique, e.g.
          "MyCompany.MyGroup.MyComponent".
        - id is a UUID for the provider. The default (and recommended) value
          is None, which will cause the id to be generated as a hash of the
          provider name using providerid_from_name.
        - group_id is a UUID of a "provider group" that the provider should
          join. Most providers are not in a group so this defaults to None.
        - throw_on_error indicates whether to raise an exception if
          EventRegister returns an error. This should usually be false for
          retail since your app should probably work normally even if the
          diagnostic logging is not working. You might want to set it to true
          for debugging.
        """

        self.__handle = ctypes.c_uint64()
        self.__traits = None
        self.name_utf8 = name_utf8
        self.id = id if id != None else providerid_from_name_bytes_utf8(name_utf8)
        self.group_id = group_id

        name_len = len(name_utf8)
        if name_len == 0:
            raise ValueError('Provider name must not be empty.', name_utf8)
        if 0 <= name_utf8.find(b'\0'):
            raise ValueError('Field name must not contain 0x00 bytes.', name_utf8)

        traits_len = 2 \
            + name_len + 1 \
            + (19 if self.group_id != None else 0) # 19 = sizeof(UINT16 + UINT8 + GUID)
        traits = (ctypes.c_byte * traits_len)()
        i = 0
        Provider.__struct_u16.pack_into(traits, i, traits_len)
        i += 2
        traits[i:i + name_len] = name_utf8
        i += name_len
        traits[i] = 0
        i += 1
        if self.group_id != None:
            # 19 = sizeof(UINT16 + UINT8 + GUID)
            # 1 = EtwProviderTraitTypeGroup
            Provider.__struct_group.pack_into(traits, i, 19, 1, self.group_id.bytes_le)
            i += 19
        self.__traits = traits

        status = Provider.__EventRegister(self.id.bytes_le, None, None, ctypes.byref(self.__handle))
        if status != 0:
            if throw_on_error: raise OSError('EventRegister error', status)
        else:
            # 2 = EventProviderSetTraits
            status = Provider.__EventSetInformation(self.__handle, 2, self.__traits, len(self.__traits))
            if status != 0 and throw_on_error: raise OSError('EventSetInformation error', status)

    def close(self) -> None:
        """Closes the provider handle."""
        if self.__handle.value != Provider.__null_handle.value:
            status = Provider.__EventUnregister(self.__handle)
            self.__handle = Provider.__null_handle
            assert status == 0

    def is_open(self) -> bool:
        """Returns true if the provider handle is open."""
        return self.__handle.value != Provider.__null_handle.value

    def is_enabled(self, level : int = 0, keyword : int = 0) -> bool:
        """Returns true if any session is listening for events from this provider.

        Use is_enabled() to skip generating events that nobody will receive:

            if provider.is_enabled(my_event_level, my_event_keyword):
                eb = EventBuilder()
                eb.reset(b'MyEventName', my_event_level, my_event_keyword)
                eb.add_bool32(b'MyFieldName', my_field_value) # as needed to add fields.
                provider.write(eb)
        """
        # This implementation of is_enabled() takes 600ns on my system.
        # An EventRegister callback would allow a Python-only implementation
        # of this, which I estimate would be about 100ns. I'm not sure the
        # improvement is worth the added complexity and reliability issues
        # associated with a callback (i.e. if you have a callback and you fail
        # to call EventUnregister when the callback goes out of scope then the
        # process will crash). If 500ns per is_enabled() call is really affecting
        # performance, it means you're calling is_enabled() far too often.
        return 0 != Provider.__EventProviderEnabled(self.__handle, level, keyword)

    def write(self,
        event : EventBuilder,
        activity_id : typing.Union[None, uuid.UUID] = None,
        related_activity_id : typing.Union[None, uuid.UUID] = None,
        ) -> int:
        """Sends an EventBuilder event to ETW (calls EventWriteTransfer)

        Throws if the EventBuilder is in an error state.

        Parameters:

        - event: An EventBuilder object with the event name and event fields.
        - activity_id: UUID, default is None. Used to correlate multiple
          events. If None, the event will use the thread activity id.
        - related_activity_id: UUID, default is None.
          Indicates the parent activity of a new activity. When starting a new
          activity, set opcode to 1 = WINEVENT_OPCODE_START, set activity_id =
          new activity, and set related_activity_id = parent activity. In all
          other cases, set related_activity_id = None.

        Returns the result of calling EventWriteTransfer. Note that some
        errors errors are unavoidable, so most retail code will ignore the
        returned result, but the returned result is useful for debugging and
        diagnosis. Common result codes include:

        - 0 = ERROR_SUCCESS: either nobody listening or event was delivered.
        - 6 = ERROR_INVALID_HANDLE: EventRegister failed, so no event written.
        - 8 = ERROR_NOT_ENOUGH_MEMORY: usually means that an event recording
              session is receiving so many events that the disk can't keep up.
        - 87 = ERROR_INVALID_PARAMETER: bug in this module? (assert)
        - 534 = ERROR_ARITHMETIC_OVERFLOW: event is too large. (assert)
        - 234 = ERROR_MORE_DATA: event recording session's buffer size is too
              small for the event.

        Typically you'll want to check provider.is_enabled() before gathering data
        for the event, creating and populating the EventBuilder, and calling
        write.
        """

        # Fill in the metadata size.
        meta = event._meta
        meta_pos = event._meta_pos
        Provider.__struct_u16.pack_into(meta, 0, meta_pos)

        # Fill in the EVENT_DATA_DESCRIPTORs.
        data_desc = Provider.__data_desc
        struct_dd = Provider.__struct_data_desc
        struct_dd.pack_into(data_desc, 0, ctypes.addressof(self.__traits), len(self.__traits), 2)
        struct_dd.pack_into(data_desc, 16, ctypes.addressof(meta), meta_pos, 1)
        struct_dd.pack_into(data_desc, 32, ctypes.addressof(event._data), event._data_pos, 0)

        status = Provider.__EventWriteTransfer(
            self.__handle, event._get_event_desc(), # _get_event_desc may throw.
            None if activity_id == None else activity_id.bytes_le,
            None if related_activity_id == None else related_activity_id.bytes_le,
            3, data_desc)

        assert status != 87, 'EventWriteTransfer ERROR_INVALID_PARAMETER (bug?)'
        assert status != 534, 'EventWriteTransfer ERROR_ARITHMETIC_OVERFLOW (event is too large)'
        return status

def _example():

    provider = Provider(b'MyCompany.MyGroup.MyComponent')
    my_field_value = True
    my_event_level = 5      # 5 = Verbose
    my_event_keyword = 0x21 # User-defined bits indicating categories.
    if provider.is_enabled(my_event_level, my_event_keyword):
        eb = EventBuilder()
        eb.reset(b'MyEventName', my_event_level, my_event_keyword)
        eb.add_bool32(b'MyFieldName', my_field_value) # as needed to add fields.
        provider.write(eb)
