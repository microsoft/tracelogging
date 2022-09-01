//! tracelogging::provider.

#![no_std]
#![warn(missing_docs)]
#![allow(clippy::needless_return)]

pub use enums::Channel;
pub use enums::InType;
pub use enums::Level;
pub use enums::Opcode;
pub use enums::OutType;
pub use guid::Guid;
pub use native::NativeImplementation;
pub use native::ProviderEnableCallback;
pub use native::NATIVE_IMPLEMENTATION;
pub use provider::Provider;
pub mod _internal;

/// Creates a static symbol representing an ETW provider that can be used with
/// `write_event!`.
///
/// # Syntax
///
/// `define_provider!(PROVIDER_SYMBOL, "ProviderName", options...);`
///
/// **Options:**
///
/// - `id("ProviderGuid")`
/// - `group_id("ProviderGroupGuid")`
///
/// # Overview
///
/// The `define_provider` macro creates a static symbol representing an ETW
/// (Event Tracing for Windows) event provider. The provider symbol can be used with the
/// `write_event` macro to send TraceLogging-encoded events to ETW.
///
/// You can think of `define_provider!(MY_PROVIDER, "MyProviderName");` as expanding
/// to code approximately like:
///
/// ```ignore
/// static MY_PROVIDER: tracelogging::Provider =
///     tracelogging::Provider::new("MyProviderName");
/// ```
///
/// > **Note:** The provider starts out unregistered. You must call
/// > `PROVIDER_SYMBOL.register()` to open the provider before using it, and you must call
/// > `PROVIDER_SYMBOL.unregister()` to close the provider before your component unloads.
/// > With the exception of `register()`, all operations on an unregistered provider are
/// > no-ops (do nothing and return immediately).
///
/// # Example
///
/// ```
/// use tracelogging as tlg;
///
/// tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
///
/// // Safety: Must call MY_PROVIDER.unregister() before module unload.
/// unsafe { MY_PROVIDER.register(); }
///
/// let message = "We're low on ice cream.";
/// tlg::write_event!(
///     MY_PROVIDER,
///     "MyWarningEvent",
///     level(Warning),
///     str("MyFieldName", message),
/// );
///
/// MY_PROVIDER.unregister();
/// ```
///
/// # Required parameters
///
/// - `PROVIDER_SYMBOL`
///
///   The symbol that will be used to refer to the provider, e.g.
///   `PROVIDER_SYMBOL.register()` or `write_event!(PROVIDER_SYMBOL, ...)`.
///
/// - `"ProviderName"`
///
///   A string literal that specifies a short human-readable name for
///   the provider. This string will be included in the events and will be a primary
///   attribute for event identification. It needs to be unique so that it does not
///   conflict with names used by other providers. It should follow a namespace
///   convention like "CompanyName.ComponentName".
///
/// # Options
///
/// - `id("GUID")`
///
///   Specifies the provider id. If the `id` option is not specified, the
///   provider id will be `Guid::from_name(provider_name)`. Most providers should use the
///   automatically-generated id so most providers do not need the `id` option.
///
///   Example: `id("80c257fb-c6bc-4538-a4c4-c7b863d46a8c")`
///
/// - `group_id("GUID")`
///
///   Specifies the provider group id. Most providers do not join a
///   provider group so most providers do not need the `group_id` option.
///
///   Example: `group_id("f73b8292-f610-4fa7-ba62-708353d162c4")`
///
/// - `debug()`
///
///   Enables a diagnostic dump of the expanded macro during compilation (for
///   non-production debugging purposes only).
#[cfg(feature = "macros")]
pub use tracelogging_macros::define_provider;

/// Sends an event to ETW via the specified provider.
///
/// # Syntax
///
/// `write_event!(PROVIDER_SYMBOL, "EventName", options and fields...);`
///
/// **Options:**
///
/// - `level(Verbose)`
/// - `keyword(0x123)`
/// - `opcode(Info)`
/// - `activity_id(&guid)`
/// - `related_id(&guid)`
/// - `task(24)`
/// - `tag(0x123)`
/// - `id_version(23, 0)`
/// - `channel(TraceLogging)`
/// - `debug()`
///
/// **Fields:**
///
/// - `u32("FieldName", &int_val)`
/// - `u32_slice("FieldName", &int_vals[..])`
/// - `str("FieldName", str_val)`
/// - `str_json("FieldName", json_str_val)`
/// - `struct("FieldName" { str("NestedField", str_val) })`
/// - ... and many more (see below)
///
/// # Overview
///
/// The `write_event` macro creates a TraceLogging-encoded event and sends it to ETW
/// (Event Tracing for Windows) using an event provider that was created by the
/// `define_provider` macro.
///
/// You can think of `write_event!(PROVIDER_SYMBOL, "EventName", options and fields...)`
/// as expanding to code that is something like the following:
///
/// ```ignore
/// if !PROVIDER_SYMBOL.enabled(event_level, event_keyword) {
///     0
/// } else {
///     EventWriteTransfer(PROVIDER_SYMBOL, options and fields...)
/// }
/// ```
///
/// > **Note:** The field values are evaluated and the event is sent to ETW only if the
/// > provider is enabled. The provider is considered enabled if the provider is
/// > registered and at least one ETW session is listening to your provider for events
/// > with the specified level and keyword.
///
/// The `write_event!` macro returns a `u32` value with a Win32 result code. If the
/// provider is disabled, `write_event` immediately returns 0 (`ERROR_SUCCESS`).
/// Otherwise, it returns the value returned by the underlying Windows
/// [EventWriteTransfer](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventwritetransfer)
/// API. This value is normally ignored in production code and is only used for debugging
/// and troubleshooting since retail code should usually treat logging APIs as
/// fire-and-forget.
/// 
/// # Limitations
/// 
/// ETW is designed for efficient handling of small events. ETW events have the following
/// limits:
/// 
/// - If the total event size (including ETW headers, provider name string, event name
///   string, field name strings, and event data) exceeds the buffer size of a logger
///   session, the event will not be delivered to that session.
/// - If the total event size (including ETW headers, provider name string, event name
///   string, field name strings, and event data) exceeds 64KB, the event will not be
///   delivered to any sessions.
/// - If the event contains more than 128 chunks of data, ETW will not be able to process
///   the event. The `write_event` macro uses one chunk for every simple field and two
///   chunks for complex fields (binary, string, and slice fields). The `write_event`
///   macro will generate a compile error if it needs more than 128 chunks. You might be
///   able to work around this limitation by merging multiple fields into one chunk using
///   "raw" field types.
/// - If the event contains more than 128 fields, the Windows Trace Decoding Helpers
///   (TDH) library will be unable to decode the event. A field is anything with a
///   "FieldName". The `write_event` macro will generate a compile error if your event
///   has more than 128 fields. You might be able to work around this limitation by using
///   arrays or by logging a series of simpler events instead of a single complex event.
/// - If a struct contains more than 127 fields, the TraceLogging encoding will be unable
///   to represent the event. A field is anything with a "FieldName". The `write_event`
///   macro will generate a compile error if a struct has more than 127 fields. You might
///   be able to work around this limitation by using arrays or by logging a series of
///   simpler events instead of a single complex event.
///
/// # Example
///
/// ```
/// use tracelogging as tlg;
///
/// tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
///
/// // Safety: Must call MY_PROVIDER.unregister() before module unload.
/// unsafe { MY_PROVIDER.register(); }
///
/// let message = "We're low on ice cream.";
/// tlg::write_event!(
///     MY_PROVIDER,
///     "MyWarningEvent",
///     level(Warning),
///     str("MyFieldName", message),
/// );
///
/// MY_PROVIDER.unregister();
/// ```
///
/// # Required parameters
///
/// - `PROVIDER_SYMBOL`
/// 
///   The symbol for the provider that will be used for sending the event to ETW.
///   This is a symbol that was created by a `define_provider!` macro.
/// 
/// - `"EventName"`
/// 
///   A string literal that specifies a short human-readable name for the event. The
///   string will be included in the event and will be a primary attribute for event
///   identification. It should be unique so that it is the resulting events will not be
///   confused with other events from providers with the same provider name.
///
/// # Options
///
/// - `level(event_level)`
/// 
///   Specifies the level (severity) of the event.
/// 
///   Level is important for event filtering so all events should specify a meaningful
///   non-zero level.
/// 
///   If the `level` option is not specified then the event's level will be Verbose. If
///   the level is specified it must be a constant `tracelogging::Level` value.
/// 
/// - `keyword(event_keyword)`
/// 
///   Specifies the keyword (category bits) of the event. The keyword is a 64-bit value
///   where each bit in the keyword corresponds to a provider-defined category. For
///   example, the "MyCompany.MyComponent" provider might define keyword bit `0x2` to
///   indicate that the event is part of a "networking" event category, so any event with
///   the `0x2` bit set in the keyword is considered as belonging to the "networking"
///   category.
/// 
///   Keyword is important for event filtering, so all events should specify a meaningful
///   non-zero keyword.
/// 
///   If the `keyword` option is not specified the event's keyword will be `0x1` to flag
///   the event as not having any assigned keyword. If the `keyword` option is specified
///   it must be a constant `u64` value. The `keyword` option can be specified more than
///   once, in which case all provided keyword values will be OR'ed together in the
///   event's keyword.
/// 
/// - `opcode(event_opcode)`
/// 
///   Specifies the opcode attribute for the event, which indicates special event
///   semantics such as "activity start" or "activity stop" that can be used by the event
///   decoder to group events together.
/// 
///   If the `opcode` option is not specified the event's opcode will be Info, indicating
///   no special semantics. If the opcode is specified it must be a constant
///   `tracelogging::Opcode` value.
/// 
/// - `activity_id(&guid)`
/// 
///   Specifies the activity id to use for the event.
/// 
///   If not specified, the event will use the current thread's thread-local activity id.
///   If specified, the value must be a reference to a `tracelogging::Guid` value.
/// 
/// - `related_id(&guid)`
/// 
///   Specifies the related activity id to use for the event. This value is normally set
///   for the activity-start event to record the parent activity of the newly-started
///   activity. This is normally left unset for all other events. 
/// 
///   If not specified, the event will not have any related activity id. If specified,
///   the value must be a reference to a `tracelogging::Guid` value.
/// 
/// - `task(event_task)`
/// 
///   Specifies the task attribute for the event. Task is a 16-bit value with
///   provider-defined semantics. Task is typically used to indicate event semantics,
///   with one or more events using a particular task value to indicate that the event
///   has the specified semantics. For example, "IPv4-Packet-Sent" and "IPv6-Packet-Sent"
///   events might be assigned the same task value.
/// 
///   If the `task` option is not specified then the event's task will be 0. If the task
///   is specified it must be a constant `u16` value.
/// 
/// - `tag(event_tag)`
/// 
///   Specifies the tag to use for the event. A tag is a 28-bit provider-defined value
///   that is available when the event is decoded. The tag's semantics are
///   provider-defined, e.g. the "MyCompany.MyComponent" provider might define tag value
///   `0x1` to mean the event contains high-priority information. Most providers do not
///   use tags so most events do not need the `tag` option.
/// 
///   If the `tag` option is not specified the event's tag will be 0. If specified, the
///   tag must be a constant `u32` value in the range 0 to 0x0FFFFFFF.
/// 
/// - `id_version(event_id, event_version)`
/// 
///   Specifies a manually-assigned numeric id for this event, along with a version
///   that indicates changes in the event schema or semantics. The version should start
///   at 0 and should be incremented each time a breaking change is made to the event,
///   e.g. when a field is removed or a field changes type. Most providers use the
///   event name for event identification so most events do not need to use the
///   `id_version` option.
/// 
///   If the `id_version` option is not specified then the event's id and version will be
///   0, indicating that no id has been assigned to the event. If id and version are
///   specified, the id must be a constant `u16` value and the version must be a constant
///   `u8` value.
/// 
/// - `channel(event_channel)`
/// 
///   Specifies the channel attribute for the event. Most events use the default
///   (TraceLogging) channel so most events do not need the `channel` option. 
/// 
///   If the `channel` option is not specified the event's channel will be TraceLogging.
///   If the channel is specified it must be a constant `tracelogging::Channel` value.
/// 
/// - `debug()`
/// 
///   Enables a diagnostic dump of the expanded macro during compilation (for
///   non-production debugging purposes only).
///
/// # Fields
///
/// Each event can have up to 128 fields. Each field has a type, a name, and a value. It
/// may also have an optional tag and an optional format override.
/// 
/// **Field syntax:** `TYPE("NAME", VALUE, tag(TAG), format(FORMAT))`
/// 
/// - `TYPE` controls the expected type of the `value` expression, the encoding that the
///   field will have in the event, and default format that the field will have when it
///   is decoded.
/// - `"NAME"` is a string literal that specifies the name of the field.
/// - `VALUE` is a Rust expression that provides the value of the field. Note that this
///   expression will be evaluated only if the event is enabled (only if at least one
///   logging session is listening for the event).
/// - `tag(TAG)` specifies an optional 28-bit "field tag" with provider-defined
///   semantics. This is usually omitted because most providers do not use field tags.
///   If not present, the field tag is 0. If present, the TAG must be a 28-bit constant
///   `u8` value in the range 0 to 0x0FFFFFFF.
/// - `format(FORMAT)` specifies an optional OutType that overrides the default format of
///   the TYPE. This is usually omitted because most valid formats are already available
///   via TYPE (e.g. you could specify a `str` type with a `Json` format, but there is
///   already a `str_json` type that has the same effect). If not present, the field's
///   format depends on the field's TYPE. If present, the FORMAT must be a constant
///   OutType value.
/// 
/// ```
/// # use tracelogging as tlg;
/// # tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
/// # unsafe { MY_PROVIDER.register(); }
/// let message = "We're low on ice cream.";
/// tlg::write_event!(MY_PROVIDER, "MyWarningEvent", level(Warning),
///     str("MyField1", message),               // No options (normal)
///     str("MyField2", message, format(Json)), // Using the format option
///     str("MyField3", message, tag(0x1234)),  // Using the tag option
///     str("MyField4", message, format(Json), tag(0x1234)), // Both options
/// );
/// # MY_PROVIDER.unregister();
/// ```
/// 
/// ## Normal field types
/// 
/// All field types expect a reference value, either `&T` or `&[T]`. All field types that
/// expect a slice `&[T]` value can also accept `&impl AsRef<[T]>`.
/// 
/// | Field Type | Rust Type | ETW Type | Notes
/// |------------|-----------|-----------|------
/// |`binary`|`&[u8]`|`Binary`|
/// |`binaryc`|`&[u8]`|`BinaryC`| (1)
/// |`bool32`|`&i32`|`Bool32`|
/// |`bool32_slice`|`&[i32]`|`Bool32`|
/// |`bool8`|`&bool`|`U8 + Boolean`|
/// |`bool8_slice`|`&[bool]`|`U8 + Boolean`|
/// |`char8_cp1252`|`&u8`|`U8 + String`|
/// |`char8_cp1252_slice`|`&[u8]`|`U8 + String`|
/// |`char16`|`&u16`|`U16 + String`|
/// |`char16_slice`|`&[u16]`|`U16 + String`|
/// |`codepointer`|`&usize`|`HexSize + CodePointer`|
/// |`codepointer_slice`|`&[usize]`|`HexSize + CodePointer`|
/// |`f32`|`&f32`|`F32`|
/// |`f32_slice`|`&[f32]`|`F32`|
/// |`f64`|`&f64`|`F64`|
/// |`f64_slice`|`&[f64]`|`F64`|
/// |`guid`|`&tracelogging::Guid`|`Guid`|
/// |`guid_slice`|`&[tracelogging::Guid]`|`Guid`|
/// |`hresult`|`&i32`|`I32 + HResult`|
/// |`hresult_slice`|`&[i32]`|`I32 + HResult`|
/// |`i8`|`&i8`|`I8`|
/// |`i8_slice`|`&[i8]`|`I8`|
/// |`i8_hex`|`&i8`|`U8 + Hex`|
/// |`i8_hex_slice`|`&[i8]`|`U8 + Hex`|
/// |`i16`|`&i16`|`I16`|
/// |`i16_slice`|`&[i16]`|`I16`|
/// |`i16_hex`|`&i16`|`U16 + Hex`|
/// |`i16_hex_slice`|`&[i16]`|`U16 + Hex`|
/// |`i32`|`&i32`|`I32`|
/// |`i32_slice`|`&[i32]`|`I32`|
/// |`i32_hex`|`&i32`|`Hex32`|
/// |`i32_hex_slice`|`&[i32]`|`Hex32`|
/// |`i64`|`&i64`|`I64`|
/// |`i64_slice`|`&[i64]`|`I64`|
/// |`i64_hex`|`&i64`|`Hex64`|
/// |`i64_hex_slice`|`&[i64]`|`Hex64`|
/// |`isize`|`&isize`|`ISize`|
/// |`isize_slice`|`&[isize]`|`ISize`|
/// |`isize_hex`|`&isize`|`HexSize`|
/// |`isize_hex_slice`|`&[isize]`|`HexSize`|
/// |`ipv4`|`&[u8; 4]`|`U32 + IPv4`|
/// |`ipv4_slice`|`&[[u8; 4]]`|`U32 + IPv4`|
/// |`ipv6`|`&[u8]`|`Binary + IPv6`|
/// |`ipv6c`|`&[u8]`|`BinaryC + IPv6`| (1)
/// |`pid`|`&u32`|`U32 + Pid`|
/// |`pid_slice`|`&[u32]`|`U32 + Pid`|
/// |`pointer`|`&usize`|`HexSize`|
/// |`pointer_slice`|`&[usize]`|`HexSize`|
/// |`port`|`&u16`|`U16 + Port`|
/// |`port_slice`|`&[u16]`|`U16 + Port`|
/// |`socketaddress`|`&[u8]`|`Binary + SocketAddress`|
/// |`socketaddressc`|`&[u8]`|`BinaryC + SocketAddress`| (1)
/// |`str`|`&[u8]`|`Str8 + Utf8`|
/// |`str_cp1252`|`&[u8]`|`Str8`|
/// |`str_json`|`&[u8]`|`Str8 + Json`|
/// |`str_xml`|`&[u8]`|`Str8 + Xml`|
/// |`str16`|`&[u16]`|`Str16`|
/// |`str16_json`|`&[u16]`|`Str16 + Json`|
/// |`str16_xml`|`&[u16]`|`Str16 + Xml`|
/// |`systemtime`|`&SystemTime`|`FileTime`| (2)
/// |`systemtime_utc`|`&SystemTime`|`FileTime + DateTimeUtc`| (2)
/// |`sz`|`&[u8]`|`Sz8 + Utf8`| (3)
/// |`sz_cp1252`|`&[u8]`|`Sz8`| (3)
/// |`sz_json`|`&[u8]`|`Sz8 + Json`| (3)
/// |`sz_xml`|`&[u8]`|`Sz8 + Xml`| (3)
/// |`sz16`|`&[u16]`|`Sz16`| (3)
/// |`sz16_json`|`&[u16]`|`Sz16 + Json`| (3)
/// |`sz16_xml`|`&[u16]`|`Sz16 + Xml`| (3)
/// |`tid`|`&u32`|`U32 + Tid`|
/// |`tid_slice`|`&[u32]`|`U32 + Tid`|
/// |`u8`|`&u8`|`U8`|
/// |`u8_slice`|`&[u8]`|`U8`|
/// |`u8_hex`|`&u8`|`U8 + Hex`|
/// |`u8_hex_slice`|`&[u8]`|`U8 + Hex`|
/// |`u16`|`&u16`|`U16`|
/// |`u16_slice`|`&[u16]`|`U16`|
/// |`u16_hex`|`&u16`|`U16 + Hex`|
/// |`u16_hex_slice`|`&[u16]`|`U16 + Hex`|
/// |`u32`|`&u32`|`U32`|
/// |`u32_slice`|`&[u32]`|`U32`|
/// |`u32_hex`|`&u32`|`Hex32`|
/// |`u32_hex_slice`|`&[u32]`|`Hex32`|
/// |`u64`|`&u64`|`U64`|
/// |`u64_slice`|`&[u64]`|`U64`|
/// |`u64_hex`|`&u64`|`Hex64`|
/// |`u64_hex_slice`|`&[u64]`|`Hex64`|
/// |`usize`|`&usize`|`USize`|
/// |`usize_hex`|`&usize`|`HexSize`|
/// |`usize_hex_slice`|`&[usize]`|`HexSize`|
/// |`usize_slice`|`&[usize]`|`USize`|
/// |`win_error`|`&u32`|`U32 + Win32Error`|
/// |`win_error_slice`|`&[u32]`|`U32 + Win32Error`|
/// |`win_filetime`|`&i64`|`FileTime`|
/// |`win_filetime_slice`|`&[i64]`|`FileTime`|
/// |`win_filetime_utc`|`&i64`|`FileTime + DateTimeUtc`|
/// |`win_filetime_utc_slice`|`&[i64]`|`FileTime + DateTimeUtc`|
/// |`win_ntstatus`|`&i32`|`Hex32 + NtStatus`|
/// |`win_ntstatus_slice`|`&[i32]`|`Hex32 + NtStatus`|
/// |`win_sid`|`&[u8]`|`Sid`| (4)
/// |`win_systemtime`|`&[u16; 8]`|`SystemTime`|
/// |`win_systemtime_slice`|`&[[u16; 8]]`|`SystemTime`|
/// |`win_systemtime_utc`|`&[u16; 8]`|`SystemTime + DateTimeUtc`|
/// |`win_systemtime_utc_slice`|`&[[u16; 8]]`|`SystemTime + DateTimeUtc`|
/// 
/// 1. The `binary` and `binaryc` encodings are the same except that the "c" type uses a
///    newer ETW encoding. The new encoding avoids the extra `FieldName.Length` field that
///    sometimes shows up for `binary` fields, but this encoding requires updated decoder
///    support and will usually only work on Windows 2018 Fall Update or later.
/// 2. The `systemtime` types will convert the provided Rust `SystemTime` value into a
///    Win32 `FILETIME` with saturation: if the `SystemTime` value is a date before 1601,
///    the logged `FILETIME` value will be 1601, and if the `SystemTime` value is a date
///    in 30828 or later, the logged `FILETIME` value will be the end of 30827.
/// 3. The `str` types are encoded as counted strings, meaning that embedded `'\0'`
///    characters are not a special case. The `sz` types are encoded as nul-terminated
///    strings, meaning that during the logging process, the `write_event` macro must
///    scan the provided value for any `'\0'` characters. This adds extra overhead to the
///    logging process, so the `str` types should be preferred over the `sz` types.
/// 4. The `win_sid` type requires an input value that is at least
///    [GetSidLength](https://docs.microsoft.com/windows/win32/api/securitybaseapi/nf-securitybaseapi-getlengthsid)
///    bytes long and will panic if the value is less than that size.
/// 
/// ## Struct field type
/// 
/// TraceLogging events can contain "struct" fields. A struct is a group of fields that are
/// logically considered a single field. A struct field has a syntax that is different from
/// the normal field type:
/// 
/// **Struct syntax:** `struct("NAME", tag(TAG), { Nested fields... })`
/// 
/// - There is no `VALUE` for a struct (its value is the nested fields).
/// - The `format(FORMAT)` option is not allowed.
/// - The `tag(TAG)` option is allowed.
/// - The nested fields are specified within curly braces `{}`.
/// - Multiple levels of nesting are supported.
/// 
/// ```
/// # use tracelogging as tlg;
/// # tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
/// # unsafe { MY_PROVIDER.register(); }
/// let message = "We're low on ice cream.";
/// tlg::write_event!(MY_PROVIDER, "MyWarningEvent", level(Warning),
///     struct("MyStruct", {
///         str("Member1", message),
///         struct("Member2", tag(0x1234), {
///             str("Nested", message),
///         }),
///     }),
/// );
/// # MY_PROVIDER.unregister();
/// ```
/// 
/// ## Raw field types
/// 
/// In certain cases, you may need capabilities not directly exposed by the normal field
/// types. For example,
/// 
/// - You might need to log an array of a variable-sized type, such as an array of
///   string.
/// - You might need to log an array of struct.
/// - You might want to log several fields in one block of data to reduce overhead.
/// 
/// In these cases, you can use the raw field types. These types are harder to use than
/// the normal field types. Using these types incorrectly can result in events that
/// cannot be decoded. To use these types correctly, you must understand how
/// TraceLogging events are encoded.
/// 
/// > **Note:** ETW stores event data tightly-packed with no padding, alignment, or size.
/// > If your field data size does not match up with your field type, the remaining
/// > fields of the event will decode incorrectly, not just the mismatched field.
/// 
/// - `raw_field("NAME", INTYPE, VALUE_BYTES, format(FORMAT), tag(TAG))`
/// 
///   The `raw_field` type allows you to add a field with direct control over the field's
///   contents. VALUE is specified as `&[u8]`, and you can specify any InType. Note that
///   the provided VALUE must include the entire field, including prefix (e.g. UInt16
///   byte count required on "Counted" fields like Binary and CountedString) or suffix
///   (e.g. NUL termination required on Sz fields).
/// 
/// - `raw_field_slice("NAME", INTYPE, VALUE, format(FORMAT), tag(TAG))`
/// 
///   The `raw_field` type allows you to add a variable-sized array field with direct
///   control over the field's contents. VALUE is specified as `&[u8]`, and you can
///   specify any InType. Note that the provided VALUE must include the entire array,
///   including the array size (UInt16 element count before the field values).
/// 
/// - `raw_meta("NAME", INTYPE, format(FORMAT), tag(TAG))`
/// 
///   The `raw_meta` type allows you to add a field definition (name, type, format, tag)
///   without adding the field's data. This allows you to specify multiple fields and
///   then provide the data for all of the fields in one blob (via `raw_data`).
/// 
/// - `raw_meta_slice("NAME", INTYPE, format(FORMAT), tag(TAG))`
/// 
///   The `raw_meta` type allows you to add an array definition (name, type, format, tag)
///   without adding the array's data. This allows you to specify multiple fields and
///   then provide the data for all of the fields in one blob (via `raw_data`).
/// 
/// - `raw_struct("NAME", FIELD_COUNT, tag(TAG))`
/// 
///   The `raw_struct` type allows you to begin a struct and directly specify the number
///   of fields in the struct with the struct's member fields specified separately (e.g.
///   via `raw_meta` or `raw_meta_slice`). Note that the FIELD_COUNT must be a constant
///   `u8` value in the range 0 to 127, and it indicates the number of subsequent logical
///   fields that will be considered to be part of the struct. In cases of nested
///   structs, a struct and its fields count as a single logical field.
/// 
/// - `raw_struct_slice("NAME", FIELD_COUNT, tag(TAG))`
/// 
///   The `raw_struct_slice` type allows you to begin an array of struct and directly
///   specify the number of fields in the struct with the struct's member fields
///   specified separately (e.g. via `raw_meta` or `raw_meta_slice`). The number of
///   elements in the array is specified as a UInt16 value immediately before the
///   array content. Note that the FIELD_COUNT must be a constant `u8` value in the
///   range 0 to 127, and it indicates the number of subsequent logical fields that will
///   be considered to be part of the struct. In cases of nested structs, a struct and
///   its fields count as a single logical field.
/// 
/// - `raw_data(VALUE)`
/// 
///   The `raw_data` type allows you to add data to the event without specifying any
///   field. This should be used together with `raw_meta` or `raw_struct` fields, where
///   the `raw_meta` or `raw_struct` field declares the field name and type, and the
///   `raw_data` field provides the data.
/// 
/// Example:
/// 
/// ```
/// # use tracelogging as tlg;
/// # tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
/// # unsafe { MY_PROVIDER.register(); }
/// tlg::write_event!(MY_PROVIDER, "MyWarningEvent", level(Warning),
/// 
///     // Make a U8 + String field containing 1 byte of data.
///     raw_field("RawChar8", U8, &[65], format(String), tag(200)),
/// 
///     // Make a U8 + String array containing u16 array-count (3) followed by 3 bytes.
///     raw_field_slice("RawChar8s", U8, &[3, 0, 65, 66, 67], format(String)),
/// 
///     // Declare a U32 + Hex field, but don't provide the data yet.
///     raw_meta("RawHex32", U32, format(Hex)),
/// 
///     // Declare a U8 + Hex array, but don't provide the data yet.
///     raw_meta_slice("RawHex8s", U8, format(Hex)),
/// 
///     // Provide the data for the previously-declared fields:
///     raw_data(&[
///         255, 0, 0, 0, // RawHex32 = 255
///         3, 0,         // RawHex8s.Length = 3
///         65, 66, 67]), // RawHex8s content = [65, 66, 67]
/// 
///     // Declare a struct with 2 fields. The next 2 logical fields in the event will be
///     // members of this struct.
///     raw_struct("RawStruct", 2),
/// 
///         // Type and data for first member of RawStruct.
///         raw_field("RawChar8", U8, &[65], format(String)),
/// 
///         // Type and data for second member of RawStruct.
///         raw_field_slice("RawChar8s", U8, &[3, 0, 65, 66, 67], format(String)),
/// 
///     // Declare a struct array with 2 fields.
///     raw_struct_slice("RawStructSlice", 2),
/// 
///         // Declare the first member of RawStructSlice. Type only (no data yet).
///         raw_meta("RawChar8", U8, format(String)),
/// 
///         // Declare the second member of RawStructSlice. Type only (no data yet).
///         raw_meta_slice("RawChar8s", U8, format(String)),
/// 
///     // Provide the data for the array of struct.
///     raw_data(&[
///         2, 0,       // RawStructSlice.Length = 2
///         48,         // RawStructSlice[0].RawChar8
///         3, 0,       // RawStructSlice[0].RawChar8s.Length = 3
///         65, 66, 67, // RawStructSlice[0].RawChar8s content
///         49,         // RawStructSlice[1].RawChar8
///         2, 0,       // RawStructSlice[1].RawChar8s.Length = 2
///         48, 49,     // RawStructSlice[1].RawChar8s content
///     ]),
/// );
/// # MY_PROVIDER.unregister();
/// ```
#[cfg(feature = "macros")]
pub use tracelogging_macros::write_event;

#[cfg(feature = "macros")]
pub use tracelogging_macros::define_provider2;

#[cfg(feature = "macros")]
pub use tracelogging_macros::write_event2;

mod descriptors;
mod enums;
mod guid;
mod native;
mod provider;

#[cfg(test)]
mod tests;
