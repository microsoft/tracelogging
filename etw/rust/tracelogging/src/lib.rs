// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![no_std]
#![warn(missing_docs)]
#![allow(clippy::needless_return)]

//! # TraceLogging-encoded ETW events
//!
//! The `tracelogging` crate provides a simple and efficient way to log
//! [TraceLogging](https://docs.microsoft.com/windows/win32/tracelogging/trace-logging-portal)
//! (manifest-free) events to
//! [ETW](https://docs.microsoft.com/windows/win32/etw/event-tracing-portal)
//! (Event Tracing for Windows). The events can be generated and collected on Windows
//! Vista or later. The events can be decoded on Windows 10 or later.
//!
//! This implementation of TraceLogging uses macros to generate event metadata at
//! compile-time, improving runtime performance and minimizing dependencies. To enable
//! compile-time metadata generation, the event schema must be specified at compile-time.
//! For example, event name and field names must be string literals, not variables.
//!
//! In rare cases, you might not know what events you want to log until runtime. For
//! example, you might be implementing a middle-layer library providing ETW support to a
//! dynamic top-layer or a scripting language like JavaScript or Python. In these cases,
//! you might use the `tracelogging_dynamic` crate instead of this crate.
//!
//! # Overview
//!
//! - Use [`define_provider!`] to create a static symbol for your [`Provider`], e.g.
//!
//!   `define_provider!(MY_PROVIDER, "MyCompany.MyComponent");`
//!
//! - During component initialization, register the provider, e.g.
//!
//!   `unsafe { MY_PROVIDER.register(); }`
//!
//!   - Safety: [`Provider::register`] is unsafe because all providers registered by a DLL
//!     **must** be properly unregistered before the DLL unloads. Since the provider is
//!     static, Rust will not automatically drop it. If you are writing an EXE, this is
//!     not a safety issue because the process exits before the EXE unloads.
//!
//! - As needed, use [`write_event!`] to send events to ETW.
//!
//! - During component shutdown, unregister the provider, e.g.
//!
//!   `MY_PROVIDER.unregister();`
//!
//! # Example
//!
//! ```
//! use tracelogging as tlg;
//!
//! // Define a static variable for the "MyCompany.MyComponent" provider.
//! tlg::define_provider!(
//!     MY_PROVIDER,              // The static symbol to use for this provider.
//!     "MyCompany.MyComponent"); // The provider's name (string literal).
//!
//! // Register the provider at module initialization. If you don't register (or if
//! // register fails) then MY_PROVIDER.enabled() will always return false, the
//! // write_event! macro will be a no-op, and MY_PROVIDER.unregister() will be a no-op.
//! // Safety: If this is a DLL, you MUST call MY_PROVIDER.unregister() before unload.
//! unsafe { MY_PROVIDER.register(); }
//!
//! // As necessary, call write_event! to send events to ETW.
//! let field1_value = "String Value";
//! let field2_value = 42u32;
//! tlg::write_event!(
//!     MY_PROVIDER,                    // The provider to use for the event.
//!     "MyEventName",                  // The event's name (string literal).
//!     level(Warning),                 // Event's severity level.
//!     keyword(0x23),                  // Event category bits.
//!     str8("Field1", field1_value),   // Add a string field to the event.
//!     u32("Field2", &field2_value),   // Add an integer field to the event.
//! );
//!
//! // Before module unload, unregister the provider.
//! MY_PROVIDER.unregister();
//! ```
//!
//! # Notes
//!
//! Field value expressions will only be evaluated if the event is enabled, i.e. only if
//! one or more ETW logging sessions are listening to the provider with filters that
//! include the level and keyword of the event.
//!
//! ETW events are limited in size (event size = headers + metadata + data). Windows will
//! ignore any event that is larger than 64KB and will ignore any event that is larger
//! than the buffer size of the recording session.
//!
//! Collect the events using Windows SDK tools like
//! [traceview](https://docs.microsoft.com/windows-hardware/drivers/devtest/traceview) or
//! [tracelog](https://docs.microsoft.com/windows-hardware/drivers/devtest/tracelog).
//! Decode the events using Windows SDK tools like
//! [traceview](https://docs.microsoft.com/windows-hardware/drivers/devtest/traceview) or
//! [tracefmt](https://docs.microsoft.com/windows-hardware/drivers/devtest/tracefmt).
//! For example, to collect events from a provider that was defined as
//! `define_provider!(MY_PROVIDER, "MyCompany.MyComponent")`:
//!
//! ```text
//! tracelog -start MyTrace -f MyTraceFile.etl -guid *MyCompany.MyComponent -level 5 -matchanykw 0x23
//! <run your program>
//! tracelog -stop MyTrace
//! tracefmt -o MyTraceData.txt MyTraceFile.etl
//! ```

/// Creates a static symbol representing an ETW provider.
///
/// `define_provider!(PROVIDER_SYMBOL, "ProviderName", options...);`
///
/// [Options:](#options)
///
/// - `id("ProviderGuid")`
/// - `group_id("ProviderGroupGuid")`
///
/// # Overview
///
/// The `define_provider!` macro creates a symbol representing an ETW (Event Tracing for
/// Windows) event provider. The symbol is a static [`Provider`] variable that can be
/// used with [`write_event!`] to send TraceLogging-encoded events to ETW.
///
/// The `PROVIDER_SYMBOL` generated by `define_provider!` should be treated as a token,
/// not a variable. When invoking [`write_event!`], use the original symbol, not a
/// reference or alias.
///
/// You can think of `define_provider!(MY_PROVIDER, "MyProviderName");` as expanding
/// to code approximately like:
///
/// ```ignore
/// static MY_PROVIDER: tracelogging::Provider =
///     tracelogging::Provider::new("MyProviderName");
/// ```
///
/// **Note:** The provider starts out unregistered. You must call
/// `MY_PROVIDER.register();` to open the provider before using it. With the exception
/// of [`Provider::register`], all operations on an unregistered provider are no-ops
/// (they will do nothing and return immediately).
///
/// # Example
///
/// ```
/// use tracelogging as tlg;
///
/// tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
///
/// // Safety: If this is a DLL, you MUST call MY_PROVIDER.unregister() before unload.
/// unsafe { MY_PROVIDER.register(); }
///
/// let message = "We're low on ice cream.";
/// tlg::write_event!(
///     MY_PROVIDER,
///     "MyWarningEvent",
///     level(Warning),
///     str8("MyFieldName", message),
/// );
///
/// MY_PROVIDER.unregister();
/// ```
///
/// # Syntax
///
/// `define_provider!(PROVIDER_SYMBOL, "ProviderName", options...);`
///
/// ## Required parameters
///
/// - `PROVIDER_SYMBOL`
///
///   The token that will be used to refer to the provider. This is used with
///   [`write_event!`] and with [Provider] methods like [`Provider::register`].
///
/// - `"ProviderName"`
///
///   A **string literal** that specifies a short human-readable name for
///   the provider. This string will be included in the events and will be a primary
///   attribute for event identification. It needs to be unique so that it does not
///   conflict with names used by other providers. It should follow a namespace
///   convention like "CompanyName.ComponentName".
///
/// ## Options
///
/// - `id("GUID")`
///
///   Specifies the ETW provider id, also known as the ETW Control GUID. If the `id`
///   option is not specified, the provider's id will be
///   `Guid::from_name(provider_name)`. Most providers should use the
///   automatically-generated id so most providers do not need to specify the `id`
///   option.
///
///   Example: `id("80c257fb-c6bc-4538-a4c4-c7b863d46a8c")`
///
/// - `group_id("GUID")`
///
///   Specifies the ETW
///   [provider group](https://docs.microsoft.com/windows/win32/etw/provider-traits) id.
///   Most providers do not join a provider group so most providers do not need to
///   specify the `group_id` option.
///
///   Example: `group_id("f73b8292-f610-4fa7-ba62-708353d162c4")`
///
/// - `debug()`
///
///   For non-production diagnostics: prints the expanded macro during compilation.
///
/// - For compability with the `eventheader` crate, certain other options may be
///   accepted and ignored.
#[cfg(feature = "macros")]
pub use tracelogging_macros::define_provider;

/// Sends an event to ETW via the specified provider.
///
/// `write_event!(PROVIDER_SYMBOL, "EventName", options and fields...);`
///
/// [Options:](#options)
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
/// [Fields:](#fields-1)
///
/// - `u32("FieldName", &int_val)`
/// - `u32_slice("FieldName", &int_vals[..])`
/// - `str8("FieldName", str_val)`
/// - `str8_json("FieldName", json_str_val)`
/// - `struct("FieldName", { str8("NestedField", str_val), ... })`
/// - [and many more...](#normal-field-types)
///
/// # Overview
///
/// The `write_event!` macro creates a TraceLogging-encoded event and sends it to ETW
/// (Event Tracing for Windows) using a [Provider] that was created by
/// [`define_provider!`].
///
/// You can think of `write_event!(MY_PROVIDER, "EventName", options and fields...)`
/// as expanding to code that is something like the following:
///
/// ```ignore
/// if !MY_PROVIDER.enabled(event_level, event_keyword) {
///     0
/// } else {
///     EventWriteTransfer(MY_PROVIDER, options and fields...)
/// }
/// ```
///
/// The `PROVIDER_SYMBOL` generated by [`define_provider!`] should be treated as a token,
/// not a variable. When invoking `write_event!`, use the original symbol, not a
/// reference or alias.
///
/// **Note:** The field value expressions are evaluated and the event is sent to ETW only
/// if the event is enabled, i.e. only if one or more ETW logging sessions are listening
/// to the provider with filters that include the level and keyword of the event.
///
/// The `write_event!` macro returns a `u32` value with a Win32 result code. If no ETW
/// logging sessions are listening for the event, `write_event!` immediately returns 0
/// (`ERROR_SUCCESS`). Otherwise, it returns the value returned by the underlying Windows
/// [EventWriteTransfer](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventwritetransfer)
/// API. Since most components treat logging APIs as fire-and-forget, this value should
/// normally be ignored in production code. It is generally used only for debugging and
/// troubleshooting.
///
/// # Limitations
///
/// ETW is optimized for efficient handling of small events. ETW events have the
/// following limits:
///
/// - If the total event size (including ETW headers, provider name string, event name
///   string, field name strings, and event data) exceeds 64KB, the event will not be
///   delivered to any sessions.
/// - If the total event size exceeds the buffer size of a logger session, the event will
///   not be delivered to that session.
/// - If the event contains more than 128 chunks of data, ETW will not be able to process
///   the event. The `write_event!` macro uses one chunk for every simple field and two
///   chunks for complex fields (binary, string, and slice fields). `write_event!` will
///   generate a compile error if it needs more than 128 chunks. You might be able to
///   work around this limitation by merging multiple fields into one chunk using
///   [raw field](#raw-fields) types.
/// - If the event contains more than 128 fields, the Windows Trace Decoding Helper
///   (TDH) library will be unable to decode the event. A field is anything with a
///   "FieldName". `write_event!` will generate a compile error if your event has more
///   than 128 fields. You might be able to work around this limitation by using arrays
///   or by logging a series of simpler events instead of a single complex event.
/// - If a struct contains more than 127 fields, the TraceLogging encoding will be unable
///   to represent the event. A field is anything with a "FieldName". The `write_event!`
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
/// // Safety: If this is a DLL, you MUST call MY_PROVIDER.unregister() before unload.
/// unsafe { MY_PROVIDER.register(); }
///
/// let message = "We're low on ice cream.";
/// tlg::write_event!(
///     MY_PROVIDER,
///     "MyWarningEvent",
///     level(Warning),
///     str8("MyFieldName", message),
/// );
///
/// MY_PROVIDER.unregister();
/// ```
///
/// # Syntax
///
/// `write_event!(PROVIDER_SYMBOL, "EventName", options and fields...);`
///
/// ## Required parameters
///
/// - `PROVIDER_SYMBOL`
///
///   The symbol for the provider that will be used for sending the event to ETW.
///   This is a symbol that was created by [`define_provider!`].
///
///   This should be the original symbol name created by [`define_provider!`], not a
///   reference or alias.
///
/// - `"EventName"`
///
///   A **string literal** that specifies a short human-readable name for the event. The
///   name will be included in the event and will be a primary attribute for event
///   identification. It should be unique so that the resulting events will not be
///   confused with other events in the same provider.
///
/// ## Options
///
/// - `level(event_level)`
///
///   Specifies the level (severity) of the event.
///
///   Level is important for event filtering so all events should specify a meaningful
///   non-zero level.
///
///   If the `level` option is not specified then the event's level will be
///   [Level::Verbose]. If the level is specified it must be a constant [Level] value.
///
/// - `keyword(event_keyword)`
///
///   Specifies the keyword (category bits) of the event.
///
///   The keyword is a 64-bit value where each bit in the keyword corresponds to a
///   provider-defined category. For example, the "MyCompany.MyComponent" provider might
///   define keyword bit `0x2` to indicate that the event is part of a "networking" event
///   category. In that case, any event in that provider with the `0x2` bit set in the
///   keyword is considered as belonging to the "networking" category.
///
///   Keyword is important for event filtering so all events should specify a meaningful
///   non-zero keyword.
///
///   If no `keyword` options are specified then the event's keyword will be `0x1` to
///   flag the event as not having any assigned keyword. If the `keyword` option is
///   specified it must be a constant `u64` value. The `keyword` option may be specified
///   more than once, in which case all provided keyword values will be OR'ed together in
///   the event's keyword.
///
/// - `opcode(event_opcode)`
///
///   Specifies the opcode attribute for the event.
///
///   The opcode indicates special event semantics such as "activity start" or "activity
///   stop" that can be used by the event decoder to group events together.
///
///   If the `opcode` option is not specified the event's opcode will be [Opcode::Info],
///   indicating no special semantics. If the opcode is specified it must be a constant
///   [Opcode] value.
///
/// - `activity_id(&guid)`
///
///   Specifies the activity id to use for the event.
///
///   If not specified, the event will use the current thread's thread-local activity id.
///   If specified, the value must be a reference to a [Guid] or a reference to a
///   `[u8; 16]`.
///
/// - `related_id(&guid)`
///
///   Specifies the related activity id to use for the event.
///
///   This value is normally set for the activity-[start](Opcode::ActivityStart) event to record the
///   parent activity of the newly-started activity. This is normally left unset for other
///   events.
///
///   If not specified, the event will not have any related activity id.
///   If specified, the value must be a reference to a [Guid] or a reference to a
///   `[u8; 16]`.
///
/// - `task(event_task)`
///
///   Specifies the task attribute for the event.
///
///   Task is a 16-bit value with provider-defined semantics. Task is typically used to
///   indicate semantic event identity, with one or more events using a particular task
///   value to indicate that the event has the specified semantics. For example, task
///   `47` might be assigned semantics "Packet Sent", and then the "IPv4-Packet-Sent" and
///   "IPv6-Packet-Sent" events might both be set to use task `47`.
///
///   If the `task` option is not specified then the event's task will be 0. If the task
///   is specified it must be a constant `u16` value.
///
/// - `tag(event_tag)`
///
///   Specifies the tag to use for the event.
///
///   A tag is a 28-bit provider-defined value that is available when the event is
///   decoded. The tag's semantics are provider-defined, e.g. the "MyCompany.MyComponent"
///   provider might define tag value `0x1` to mean the event contains high-priority
///   information. Most providers do not use tags so most events do not need to specify
///   the `tag` option.
///
///   If the `tag` option is not specified the event's tag will be 0. If specified, the
///   tag must be a constant `u32` value in the range 0 to 0x0FFFFFFF.
///
/// - `id_version(event_id, event_version)`
///
///   Specifies a manually-assigned numeric id for this event, along with a version
///   that indicates changes in the event schema or semantics.
///
///   Most providers use the event name for event identification so most events do not
///   need to specify the `id_version` option.
///
///   The version should start at 0 and should be incremented each time a breaking change
///   is made to the event, e.g. when a field is removed or a field changes type.
///
///   If the `id_version` option is not specified then the event's id and version will be
///   0, indicating that no id has been assigned to the event. If id and version are
///   specified, the id must be a constant `u16` value and the version must be a constant
///   `u8` value.
///
/// - `channel(event_channel)`
///
///   Specifies the channel attribute for the event.
///
///   Most events use the default channel (TraceLogging) so most events do not need to
///   specify the `channel` option.
///
///   If the `channel` option is not specified the event's channel will be
///   [Channel::TraceLogging]. If the channel is specified it must be a constant
///   [Channel] value.
///
/// - `debug()`
///
///   For non-production diagnostics: prints the expanded macro during compilation.
///
/// ## Fields
///
/// Event content is provided in fields. Each field is added to the event with a field
/// type.
///
/// There are three categories of field types:
///
/// - [Normal field types](#normal-fields) add a field to the event with a value such as
///   an integer, float, string, slice of i32, [etc.](#normal-field-types)
/// - [The struct field type](#struct-fields) adds a field to the event that contains a group
///   of other fields.
/// - [Raw field types](#raw-fields) directly add unchecked data (field content) and/or
///   metadata (field name and type information) to the event. They are used in advanced
///   scenarios to optimize event generation or to log complex data types that the other
///   field categories cannot handle.
///
/// ### Normal fields
///
/// All normal fields have a type, a name, and a value reference. They may optionally
/// specify a tag and/or a format.
///
/// **Normal field syntax:** `TYPE("NAME", VALUE_REF, tag(TAG), format(FORMAT))`
///
/// - `TYPE` controls the expected type of the `VALUE_REF` expression,
///   the ETW encoding that the field will use in the event, and default format that the
///   field will have when it is decoded. TYPEs include `u32`, `str8`, `str16`,
///   `f32_slice` and [many others](#normal-field-types).
///
/// - `"NAME"` is a string literal that specifies the name of the field.
///
/// - `VALUE_REF` is a Rust expression that provides a reference to the value of the
///   field.
///
///   Field types that expect a slice `&[T]` type will also accept types that implement
///   the [`AsRef<[T]>`](AsRef) trait. For example, the `str8` field types expect a
///   `&[u8]` but will also accept `&str` or `&String` because those types implement
///   `AsRef<[u8]>`.
///
///   The field value expression will be evaluated only if the event is enabled, i.e.
///   only if at least one logging session is listening to the provider and has filtering
///   that includes this event's level and keyword.
///
/// - `tag(TAG)` specifies a 28-bit "field tag" with provider-defined semantics.
///
///   This is usually omitted because most providers do not use field tags.
///
///   If not present, the field tag is `0`. If present, the TAG must be a 28-bit constant
///   `u32` value in the range `0` to `0x0FFFFFFF`.
///
/// - `format(FORMAT)` specifies an [OutType] that overrides the format that would
///   normally apply for the given `TYPE`.
///
///   This is usually omitted because most valid formats are available without the use of
///   the format option. For example, you could specify a `str8` type with a `Json`
///   format, but this is unnecessary because there is already a `str8_json` type that has
///   the same effect.
///
///   If not present, the field's format depends on the field's `TYPE`. If present, the
///   FORMAT must be a constant [OutType] value.
///
/// Example:
///
/// ```
/// # use tracelogging as tlg;
/// # tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
/// let message = "We're low on ice cream.";
/// tlg::write_event!(
///     MY_PROVIDER,
///     "MyWarningEvent",
///     level(Warning),
///     str8("MyField1", message),               // No options (normal)
///     str8("MyField2", message, format(Json)), // Using the format option
///     str8("MyField3", message, tag(0x1234)),  // Using the tag option
///     str8("MyField4", message, format(Json), tag(0x1234)), // Both options
/// );
/// ```
///
/// ### Normal field types
///
/// | Field Type | Rust Type | ETW Type
/// |------------|-----------|---------
/// | `binary` | `&[u8]` | [`Binary`](InType::Binary)
/// | `binaryc` [^binaryc] | `&[u8]` | [`BinaryC`](InType::BinaryC)
/// | `bool8` | `&bool` | [`U8`](InType::U8) + [`Boolean`](OutType::Boolean)
/// | `bool8_slice` | `&[bool]` | [`U8`](InType::U8) + [`Boolean`](OutType::Boolean)
/// | `bool32` | `&i32` | [`Bool32`](InType::Bool32)
/// | `bool32_slice` | `&[i32]` | [`Bool32`](InType::Bool32)
/// | `char8_cp1252` | `&u8` | [`U8`](InType::U8) + [`String`](OutType::String)
/// | `char8_cp1252_slice` | `&[u8]` | [`U8`](InType::U8) + [`String`](OutType::String)
/// | `char16` | `&u16` | [`U16`](InType::U16) + [`String`](OutType::String)
/// | `char16_slice` | `&[u16]` | [`U16`](InType::U16) + [`String`](OutType::String)
/// | `codepointer` | `&usize` | [`HexSize`](InType::HexSize) + [`CodePointer`](OutType::CodePointer)
/// | `codepointer_slice` | `&[usize]` | [`HexSize`](InType::HexSize) + [`CodePointer`](OutType::CodePointer)
/// | `cstr8` [^cstr] | `&[u8]` | [`CStr8`](InType::CStr8) + [`Utf8`](OutType::Utf8)
/// | `cstr8_cp1252` [^cstr] | `&[u8]` | [`CStr8`](InType::CStr8)
/// | `cstr8_json` [^cstr] | `&[u8]` | [`CStr8`](InType::CStr8) + [`Json`](OutType::Json)
/// | `cstr8_xml` [^cstr] | `&[u8]` | [`CStr8`](InType::CStr8) + [`Xml`](OutType::Xml)
/// | `cstr16` [^cstr] | `&[u16]` | [`CStr16`](InType::CStr16)
/// | `cstr16_json` [^cstr] | `&[u16]` | [`CStr16`](InType::CStr16) + [`Json`](OutType::Json)
/// | `cstr16_xml` [^cstr] | `&[u16]` | [`CStr16`](InType::CStr16) + [`Xml`](OutType::Xml)
/// | `errno` [^errno] | `&i32` | [`I32`](InType::I32)
/// | `errno_slice` [^errno] | `&[i32]` | [`I32`](InType::I32)
/// | `f32` | `&f32` | [`F32`](InType::F32)
/// | `f32_slice` | `&[f32]` | [`F32`](InType::F32)
/// | `f64` | `&f64` | [`F64`](InType::F64)
/// | `f64_slice` | `&[f64]` | [`F64`](InType::F64)
/// | `guid` | `&tracelogging::Guid` | [`Guid`](InType::Guid)
/// | `guid_slice` | `&[tracelogging::Guid]` | [`Guid`](InType::Guid)
/// | `hresult` | `&i32` | [`I32`](InType::I32) + [`HResult`](OutType::HResult)
/// | `hresult_slice` | `&[i32]` | [`I32`](InType::I32) + [`HResult`](OutType::HResult)
/// | `i8` | `&i8` | [`I8`](InType::I8)
/// | `i8_slice` | `&[i8]` | [`I8`](InType::I8)
/// | `i8_hex` | `&i8` | [`U8`](InType::U8) + [`Hex`](OutType::Hex)
/// | `i8_hex_slice` | `&[i8]` | [`U8`](InType::U8) + [`Hex`](OutType::Hex)
/// | `i16` | `&i16` | [`I16`](InType::I16)
/// | `i16_slice` | `&[i16]` | [`I16`](InType::I16)
/// | `i16_hex` | `&i16` | [`U16`](InType::U16) + [`Hex`](OutType::Hex)
/// | `i16_hex_slice` | `&[i16]` | [`U16`](InType::U16) + [`Hex`](OutType::Hex)
/// | `i32` | `&i32` | [`I32`](InType::I32)
/// | `i32_slice` | `&[i32]` | [`I32`](InType::I32)
/// | `i32_hex` | `&i32` | [`Hex32`](InType::Hex32)
/// | `i32_hex_slice` | `&[i32]` | [`Hex32`](InType::Hex32)
/// | `i64` | `&i64` | [`I64`](InType::I64)
/// | `i64_slice` | `&[i64]` | [`I64`](InType::I64)
/// | `i64_hex` | `&i64` | [`Hex64`](InType::Hex64)
/// | `i64_hex_slice` | `&[i64]` | [`Hex64`](InType::Hex64)
/// | `ipv4` | `&[u8; 4]` | [`U32`](InType::U32) + [`IPv4`](OutType::IPv4)
/// | `ipv4_slice` | `&[[u8; 4]]` | [`U32`](InType::U32) + [`IPv4`](OutType::IPv4)
/// | `ipv6` | `&[u8; 16]` | [`Binary`](InType::Binary) + [`IPv6`](OutType::IPv6)
/// | `ipv6c` [^binaryc] | `&[u8; 16]` | [`BinaryC`](InType::BinaryC) + [`IPv6`](OutType::IPv6)
/// | `isize` | `&isize` | [`ISize`](InType::ISize)
/// | `isize_slice` | `&[isize]` | [`ISize`](InType::ISize)
/// | `isize_hex` | `&isize` | [`HexSize`](InType::HexSize)
/// | `isize_hex_slice` | `&[isize]` | [`HexSize`](InType::HexSize)
/// | `pid` | `&u32` | [`U32`](InType::U32) + [`Pid`](OutType::Pid)
/// | `pid_slice` | `&[u32]` | [`U32`](InType::U32) + [`Pid`](OutType::Pid)
/// | `pointer` | `&usize` | [`HexSize`](InType::HexSize)
/// | `pointer_slice` | `&[usize]` | [`HexSize`](InType::HexSize)
/// | `port` | `&u16` | [`U16`](InType::U16) + [`Port`](OutType::Port)
/// | `port_slice` | `&[u16]` | [`U16`](InType::U16) + [`Port`](OutType::Port)
/// | `socketaddress` | `&[u8]` | [`Binary`](InType::Binary) + [`SocketAddress`](OutType::SocketAddress)
/// | `socketaddressc` [^binaryc] | `&[u8]` | [`BinaryC`](InType::BinaryC) + [`SocketAddress`](OutType::SocketAddress)
/// | `str8` | `&[u8]` | [`Str8`](InType::Str8) + [`Utf8`](OutType::Utf8)
/// | `str8_cp1252` | `&[u8]` | [`Str8`](InType::Str8)
/// | `str8_json` | `&[u8]` | [`Str8`](InType::Str8) + [`Json`](OutType::Json)
/// | `str8_xml` | `&[u8]` | [`Str8`](InType::Str8) + [`Xml`](OutType::Xml)
/// | `str16` | `&[u16]` | [`Str16`](InType::Str16)
/// | `str16_json` | `&[u16]` | [`Str16`](InType::Str16) + [`Json`](OutType::Json)
/// | `str16_xml` | `&[u16]` | [`Str16`](InType::Str16) + [`Xml`](OutType::Xml)
/// | `systemtime` [^systemtime] | `&std::time::SystemTime` | [`FileTime`](InType::FileTime)
/// | `tid` | `&u32` | [`U32`](InType::U32) + [`Tid`](OutType::Tid)
/// | `tid_slice` | `&[u32]` | [`U32`](InType::U32) + [`Tid`](OutType::Tid)
/// | `time32` [^time] | `&i32` | [`FileTime`](InType::FileTime)
/// | `time64` [^time] | `&i64` | [`FileTime`](InType::FileTime)
/// | `u8` | `&u8` | [`U8`](InType::U8)
/// | `u8_slice` | `&[u8]` | [`U8`](InType::U8)
/// | `u8_hex` | `&u8` | [`U8`](InType::U8) + [`Hex`](OutType::Hex)
/// | `u8_hex_slice` | `&[u8]` | [`U8`](InType::U8) + [`Hex`](OutType::Hex)
/// | `u16` | `&u16` | [`U16`](InType::U16)
/// | `u16_slice` | `&[u16]` | [`U16`](InType::U16)
/// | `u16_hex` | `&u16` | [`U16`](InType::U16) + [`Hex`](OutType::Hex)
/// | `u16_hex_slice` | `&[u16]` | [`U16`](InType::U16) + [`Hex`](OutType::Hex)
/// | `u32` | `&u32` | [`U32`](InType::U32)
/// | `u32_slice` | `&[u32]` | [`U32`](InType::U32)
/// | `u32_hex` | `&u32` | [`Hex32`](InType::Hex32)
/// | `u32_hex_slice` | `&[u32]` | [`Hex32`](InType::Hex32)
/// | `u64` | `&u64` | [`U64`](InType::U64)
/// | `u64_slice` | `&[u64]` | [`U64`](InType::U64)
/// | `u64_hex` | `&u64` | [`Hex64`](InType::Hex64)
/// | `u64_hex_slice` | `&[u64]` | [`Hex64`](InType::Hex64)
/// | `usize` | `&usize` | [`USize`](InType::USize)
/// | `usize_slice` | `&[usize]` | [`USize`](InType::USize)
/// | `usize_hex` | `&usize` | [`HexSize`](InType::HexSize)
/// | `usize_hex_slice` | `&[usize]` | [`HexSize`](InType::HexSize)
/// | `win_error` | `&u32` | [`U32`](InType::U32) + [`Win32Error`](OutType::Win32Error)
/// | `win_error_slice` | `&[u32]` | [`U32`](InType::U32) + [`Win32Error`](OutType::Win32Error)
/// | `win_filetime` | `&i64` | [`FileTime`](InType::FileTime)
/// | `win_filetime_slice` | `&[i64]` | [`FileTime`](InType::FileTime)
/// | `win_ntstatus` | `&i32` | [`Hex32`](InType::Hex32) + [`NtStatus`](OutType::NtStatus)
/// | `win_ntstatus_slice` | `&[i32]` | [`Hex32`](InType::Hex32) + [`NtStatus`](OutType::NtStatus)
/// | `win_sid` [^sid] | `&[u8]` | [`Sid`](InType::Sid)
/// | `win_systemtime` | `&[u16; 8]` | [`SystemTime`](InType::SystemTime)
/// | `win_systemtime_slice` | `&[[u16; 8]]` | [`SystemTime`](InType::SystemTime)
/// | `win_systemtime_utc` | `&[u16; 8]` | [`SystemTime`](InType::SystemTime) + [`DateTimeUtc`](OutType::DateTimeUtc)
/// | `win_systemtime_utc_slice` | `&[[u16; 8]]` | [`SystemTime`](InType::SystemTime) + [`DateTimeUtc`](OutType::DateTimeUtc)
///
/// [^binaryc]: The `...` and `...c` types are the same except that the `...c` types use
/// a newer `InType::BinaryC` ETW encoding. The `BinaryC` encoding avoids the extra
/// `FieldName.Length` field that sometimes shows up for `InType::Binary` fields. This
/// new encoding requires updated decoder support so it may not work with older ETW
/// decoding tools.
///
/// [^cstr]: The `cstrN` types use a `0`-terminated `InType::CStrN` string encoding in
/// the event. If the provided field value contains any `'\0'` characters then the event
/// will include the value up to the first `'\0'`; otherwise the event will include the
/// entire value. There is a small runtime overhead for locating the first `0` in the
/// string. To avoid the overhead and to ensure you log your entire string (including any
/// `'\0'` characters), prefer the `str` types (counted strings) over the `cstr` types
/// (`0`-terminated strings) unless you specifically need a `0`-terminated ETW encoding.
///
/// [^errno]: The `errno` type is intended for use with C-style `errno` error codes. On
/// Windows, the `errno` type behaves exactly like the `i32` type.
///
/// [^systemtime]: When logging `systemtime` types, `write_event!` will convert the
/// provided `std::time::SystemTime` value into a Win32
/// [`FILETIME`](https://docs.microsoft.com/windows/win32/api/minwinbase/ns-minwinbase-filetime),
/// saturating if the value is out of the range that
/// [`FileTimeToSystemTime`](https://docs.microsoft.com/windows/win32/api/timezoneapi/nf-timezoneapi-filetimetosystemtime)
/// can handle: if the `SystemTime` value is a date before 1601, the logged `FILETIME`
/// value will be the start of 1601, and if the `SystemTime` value is a date after 30827,
/// the logged `FILETIME` value will be the end of 30827.
///
/// [^sid]: The `win_sid` type requires an input byte-slice value that is at least
/// [`GetSidLength(value_bytes)`](https://docs.microsoft.com/windows/win32/api/securitybaseapi/nf-securitybaseapi-getlengthsid)
/// =  `value_bytes[1] * 4 + 8` bytes long. `write_event!` will panic if the value is
/// smaller than that size.
///
/// [^time]: When logging `time32` and `time64` types, `write_event!` assumes that the
/// provided `i32` or `i64` value is the number of seconds since 1970 (i.e. a `time_t`)
/// and will convert the value into a Win32
/// [`FILETIME`](https://docs.microsoft.com/windows/win32/api/minwinbase/ns-minwinbase-filetime),
/// saturating if the value is out of the range that
/// [`FileTimeToSystemTime`](https://docs.microsoft.com/windows/win32/api/timezoneapi/nf-timezoneapi-filetimetosystemtime)
/// can handle: if an `i64` value is a date before 1601, the logged `FILETIME`
/// value will be the start of 1601, and if the `i64` value is a date after 30827,
/// the logged `FILETIME` value will be the end of 30827.
///
/// ### Struct fields
///
/// A struct is a group of fields that are logically considered a single field.
///
/// Struct fields have type `struct`, a name, and a set of nested fields enclosed in
/// braces `{ ... }`. They may optionally specify a tag.
///
/// **Struct field syntax:** `struct("NAME", tag(TAG), { FIELDS... })`
///
/// - `"NAME"` is a string literal that specifies the name of the field.
///
/// - `tag(TAG)` specifies a 28-bit "field tag" with provider-defined semantics.
///
///   This is usually omitted because most providers do not use field tags.
///
///   If not present, the field tag is `0`. If present, the TAG must be a 28-bit constant
///   `u32` value in the range `0` to `0x0FFFFFFF`.
///
/// - `{ FIELDS... }` is a list of other fields that will be considered to be part of
///   this field. This list may include normal fields, struct fields, and non-struct raw
///   fields.
///
/// Example:
///
/// ```
/// # use tracelogging as tlg;
/// # tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
/// let message = "We're low on ice cream.";
/// tlg::write_event!(
///     MY_PROVIDER,
///     "MyWarningEvent",
///     level(Warning),
///     str8("RootField1", message),
///     str8("RootField2", message),
///     struct("RootField3", {
///         str8("MemberField1", message),
///         str8("MemberField2", message),
///         struct("MemberField3", tag(0x1234), {
///             str8("NestedField1", message),
///             str8("NestedField2", message),
///         }),
///         str8("MemberField4", message),
///     }),
///     str8("RootField4", message),
/// );
/// ```
///
/// ### Raw fields
///
/// *Advanced:* In certain cases, you may need capabilities not directly exposed by the
/// normal field types. For example,
///
/// - You might need to log an array of a variable-sized type, such as an array of
///   string.
/// - You might need to log an array of struct.
/// - You might want to log several fields in one block of data to reduce overhead.
///
/// In these cases, you can use the raw field types. These types are harder to use than
/// the normal field types. Using these types incorrectly can result in events that
/// cannot be decoded. To use these types correctly, you must understand how
/// TraceLogging events are encoded. `write_event!` does not verify that the provided
/// field types or field data are valid.
///
/// **Note:** ETW stores event data tightly-packed with no padding, alignment, or size.
/// If your field data size does not match up with your field type, the remaining
/// fields of the event will decode incorrectly, not just the mismatched field.
///
/// Each raw field type has unique syntax and capabilities. However, in all cases,
/// the `format` and `tag` options have the same significance as in normal fields and may
/// be omitted if not needed. If omitted, `tag` defaults to `0` and `format` defaults to
/// [OutType::Default].
///
/// Raw field data is always specified as `&[u8]`. The provided VALUE_BYTES must include
/// the entire field, including prefix (e.g. `u16` byte count prefix required on
/// "Counted" fields like [InType::Binary] and [InType::Str8]) or suffix (e.g. `'\0'`
/// termination required on [InType::CStr8] fields).
///
/// - `raw_field("NAME", INTYPE, VALUE_BYTES, format(FORMAT), tag(TAG))`
///
///   The `raw_field` type allows you to add a field with direct control over the field's
///   contents. VALUE_BYTES is specified as `&[u8]` and you can specify any [InType].
///
/// - `raw_field_slice("NAME", INTYPE, VALUE_BYTES, format(FORMAT), tag(TAG))`
///
///   The `raw_field` type allows you to add a variable-sized array field with direct
///   control over the field's contents. VALUE_BYTES is specified as `&[u8]` and you can
///   specify any [InType]. Note that the provided VALUE_BYTES must include the entire
///   array, including the array element count, which is a `u16` element count
///   immediately before the field values.
///
/// - `raw_meta("NAME", INTYPE, format(FORMAT), tag(TAG))`
///
///   The `raw_meta` type allows you to add a field definition (name, intype, format,
///   tag) without immediately adding the field's data. This allows you to specify
///   multiple `raw_meta` fields and then provide the data for all of the fields via one
///   or more `raw_data` fields before or after the corresponding `raw_meta` fields.
///
/// - `raw_meta_slice("NAME", INTYPE, format(FORMAT), tag(TAG))`
///
///   The `raw_meta_slice` type allows you to add a variable-length array field
///   definition (name, type, format, tag) without immediately adding the array's data.
///   This allows you to specify multiple `raw_meta` fields and then provide the data for
///   all of the fields via one or more `raw_data` fields (before or after the
///   corresponding `raw_meta` fields).
///
/// - `raw_struct("NAME", FIELD_COUNT, tag(TAG))`
///
///   The `raw_struct` type allows you to begin a struct and directly specify the number
///   of fields in the struct. The struct's member fields are specified separately (e.g.
///   via `raw_meta` or `raw_meta_slice`).
///
///   Note that the FIELD_COUNT must be a constant `u8` value in the range 0 to 127. It
///   indicates the number of subsequent logical fields that will be considered to be
///   part of the struct. In cases of nested structs, a struct and its fields count as a
///   single logical field.
///
/// - `raw_struct_slice("NAME", FIELD_COUNT, tag(TAG))`
///
///   The `raw_struct_slice` type allows you to begin a variable-length array-of-struct
///   and directly specify the number of fields in the struct. The struct's member fields
///   are specified separately (e.g. via `raw_meta` or `raw_meta_slice`).
///
///   The number of elements in the array is specified as a `u16` value immediately
///   before the array content.
///
///   Note that the FIELD_COUNT must be a constant `u8` value in the range 0 to 127. It
///   indicates the number of subsequent logical fields that will be considered to be
///   part of the struct. In cases of nested structs, a struct and its fields count as a
///   single logical field.
///
/// - `raw_data(VALUE_BYTES)`
///
///   The `raw_data` type allows you to add data to the event without specifying any
///   field. This should be used together with `raw_meta` or `raw_meta_slice` fields,
///   where the `raw_meta` or `raw_meta_slice` fields declare the field names and types
///   and the `raw_data` field(s) provide the corresponding data (including any array
///   element counts).
///
///   Note that TraceLogging events contain separate sections for metadata and data.
///   The `raw_meta` fields add to the compile-time-constant metadata section and the
///   `raw_data` fields add to the runtime-variable data section. As a result, it doesn't
///   matter whether the `raw_data` comes before or after the corresponding `raw_meta`.
///   In addition, you can use one `raw_data` to supply the data for any number of
///   fields or you can use multiple `raw_data` fields to supply the data for one field.
///
/// Example:
///
/// ```
/// # use tracelogging as tlg;
/// # tlg::define_provider!(MY_PROVIDER, "MyCompany.MyComponent");
/// tlg::write_event!(MY_PROVIDER, "MyWarningEvent", level(Warning),
///
///     // Make a U8 + String field containing 1 byte of data.
///     raw_field("RawChar8", U8, &[65], format(String), tag(200)),
///
///     // Make a U8 + String array containing u16 array-count (3) followed by 3 bytes.
///     raw_field_slice("RawChar8s", U8, &[
///         3, 0,       // RawChar8s.Length = 3
///         65, 66, 67, // RawChar8s content = [65, 66, 67]
///     ], format(String)),
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
/// ```
#[cfg(feature = "macros")]
pub use tracelogging_macros::write_event;

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
pub mod changelog;

/// Converts a
/// [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
/// into a Windows
/// [`FILETIME`](https://learn.microsoft.com/windows/win32/api/minwinbase/ns-minwinbase-filetime)
/// `i64` value.
/// (Usually not needed - the `systemtime` field type does this automatically.)
///
/// This macro will convert the provided `SystemTime` value into a Win32
/// [`FILETIME`](https://docs.microsoft.com/windows/win32/api/minwinbase/ns-minwinbase-filetime),
/// saturating if the value is out of the range that
/// [`FileTimeToSystemTime`](https://docs.microsoft.com/windows/win32/api/timezoneapi/nf-timezoneapi-filetimetosystemtime)
/// can handle: if the `SystemTime` value is a date before year 1601, the returned
/// `FILETIME` value will be the start of 1601, and if the `SystemTime` value is a date
/// after year 30827, the returned `FILETIME` value will be the end of 30827.
///
/// The returned `i64` value can be used with [`write_event!`] via the `win_filetime`
/// and `win_filetime_slice` field types. As an alternative, you can use the `systemtime`
/// field type, which will automatically convert the provided
/// `std::time::SystemTime` value into a `FILETIME` before writing the event to ETW.
///
/// Note: `win_filetime_from_systemtime` is implemented as a macro because this crate is
/// `[no_std]`. Implementing this via a function would require this crate to reference
/// `std::time::SystemTimeError`.
#[macro_export]
macro_rules! win_filetime_from_systemtime {
    // Keep in sync with tracelogging_dynamic::win_filetime_from_systemtime.
    // The implementation is duplicated to allow for different doc comments.
    ($time:expr) => {
        match $time.duration_since(::std::time::SystemTime::UNIX_EPOCH) {
            Ok(dur) => ::tracelogging::_internal::filetime_from_duration_after_1970(dur),
            Err(err) => {
                ::tracelogging::_internal::filetime_from_duration_before_1970(err.duration())
            }
        }
    };
}

mod descriptors;
mod enums;
mod guid;
mod native;
mod provider;
