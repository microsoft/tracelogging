// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![no_std]
#![warn(missing_docs)]
#![allow(clippy::needless_return)]

//! # TraceLoggingDynamic for Rust
//!
//! `tracelogging_dynamic` provides a flexible way to log
//! [TraceLogging](https://docs.microsoft.com/windows/win32/tracelogging/trace-logging-portal)
//! (manifest-free) events to
//! [ETW](https://docs.microsoft.com/windows/win32/etw/event-tracing-portal)
//! (Event Tracing for Windows). The events can be generated and collected on Windows
//! Vista or later. The events can be decoded on Windows 10 or later.
//!
//! This "dynamic" implementation of TraceLogging is more flexible than the
//! implementation in the [`tracelogging`] crate. For example, it supports
//! runtime-defined schema and can easily log arrays of strings. However, it is harder to
//! use, it has higher runtime costs, and it depends on the `alloc` crate. This dynamic
//! implementation is intended for use only when the set of events cannot be determined
//! at compile-time. For example, `traceloggingdynamic` might be used to implement a
//! middle-layer library providing ETW support to a scripting language like JavaScript or
//! Python. In other cases, use the [`tracelogging`] crate instead of this crate.
//!
//! # Overview
//!
//! - Create a pinned [Provider] object, e.g.

//!   - `let provider = pin!(Provider::new("MyCompany.MyComponent", &tld::Provider::options()));`

//!   - `let provider = Box::pin(Provider::new("MyCompany.MyComponent", &tld::Provider::options()));`
//!
//! - Call [Provider::register] to open the connection to ETW, e.g.
//!
//!   `unsafe { provider.as_ref().register(); }`
//!
//!   - Safety: [Provider::register] is unsafe because a registered provider **must** be
//!     properly unregistered. This happens automatically when the provider is dropped,
//!     so you usually don't need to worry about this, but it can be an issue in cases
//!     where dropping does not occur such as with static variables.
//!
//! - As needed, use an [EventBuilder] to construct and write events.
//!
//! - The provider will automatically unregister when it is dropped. You can manually call
//!   [Provider::unregister] if you want to unregister sooner or if the provider is a
//!   static variable.
//!
//! # Example
//!
//! ```
//! use tracelogging_dynamic as tld;
//!
//! // Pinning is required because the register() method sets up a callback with ETW.
//! let provider = Box::pin(tld::Provider::new(
//!     "MyCompany.MyComponent",
//!     &tld::Provider::options()));
//!
//! // Register the provider. If you don't register (or if register fails) then enabled()
//! // will always return false and write() will be a  no-op.
//! unsafe {
//!     provider.as_ref().register();
//! }
//!
//! // If provider is not enabled for a given level + keyword, the write() call will do
//! // nothing. Check enabled(level, keyword) before building the event so we don't waste
//! // time on an event that nobody will receive.
//! let my_event_level = tld::Level::Verbose; // Severity level.
//! let my_event_keyword = 0x123; // User-defined category bits.
//! if provider.enabled(my_event_level, my_event_keyword) {
//!     let field1_value = "FieldValue";
//!     let field2_value = b'A';
//!     // Create and write an event with two fields:
//!     tld::EventBuilder::new()
//!         // Most events specify 0 for event tag.
//!         .reset("MyEventName", my_event_level, my_event_keyword, 0)
//!         // Most fields use Default for event format and 0 for field tag.
//!         .add_str8("FieldName1", field1_value, tld::OutType::Default, 0)
//!         .add_u8("FieldName2", field2_value, tld::OutType::String, 0)
//!         // If activity_id is None, event uses the current thread's activity.
//!         // If related_id is None, event will not have a related activity.
//!         .write(&provider, None, None);
//! }
//! ```
//!
//! # Notes
//!
//! The [EventBuilder] object is reusable. You may get a small performance benefit by
//! reusing an EventBuilder object for multiple events rather than using a new
//! EventBuilder for each event.
//!
//! ETW events are limited in size (event size = headers + metadata + data). Windows will
//! ignore any event that is larger than 64KB and will ignore any event that is larger
//! than the buffer size of the recording session.
//!
//! Most ETW decoding tools are unable to decode an event with more than 128 fields.
//!
//! Collect the events using Windows SDK tools like
//! [traceview](https://docs.microsoft.com/windows-hardware/drivers/devtest/traceview) or
//! [tracelog](https://docs.microsoft.com/windows-hardware/drivers/devtest/tracelog).
//! Decode the events using Windows SDK tools like
//! [traceview](https://docs.microsoft.com/windows-hardware/drivers/devtest/traceview) or
//! [tracefmt](https://docs.microsoft.com/windows-hardware/drivers/devtest/tracefmt).
//! For example, to collect events from a provider that was registered as
//! `provider.register("MyCompany.MyComponent", ...)`:
//!
//! ```text
//! tracelog -start MyTrace -f MyTraceFile.etl -guid *MyCompany.MyComponent -level 5 -matchanykw 0xf
//! <run your program>
//! tracelog -stop MyTrace
//! tracefmt -o MyTraceData.txt MyTraceFile.etl
//! ```

// Re-exports from tracelogging:
pub use tracelogging::Channel;
pub use tracelogging::Guid;
pub use tracelogging::InType;
pub use tracelogging::Level;
pub use tracelogging::NativeImplementation;
pub use tracelogging::Opcode;
pub use tracelogging::OutType;
pub use tracelogging::ProviderEnableCallback;
pub use tracelogging::NATIVE_IMPLEMENTATION;

// Exports from tracelogging_dynamic:
pub use builder::EventBuilder;
pub use provider::Provider;
pub use provider::ProviderOptions;

pub mod changelog;

/// Converts a
/// [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
/// into a Windows
/// [`FILETIME`](https://learn.microsoft.com/windows/win32/api/minwinbase/ns-minwinbase-filetime)
/// `i64` value.
///
/// This macro will convert the provided `SystemTime` value into a Win32
/// [`FILETIME`](https://docs.microsoft.com/windows/win32/api/minwinbase/ns-minwinbase-filetime),
/// saturating if the value is out of the range that
/// [`FileTimeToSystemTime`](https://docs.microsoft.com/windows/win32/api/timezoneapi/nf-timezoneapi-filetimetosystemtime)
/// can handle: if the `SystemTime` value is a date before year 1601, the returned
/// `FILETIME` value will be the start of 1601, and if the `SystemTime` value is a date
/// after year 30827, the returned `FILETIME` value will be the end of 30827.
///
/// The returned `i64` value can be used with [`EventBuilder::add_filetime`] and
/// [`EventBuilder::add_filetime_sequence`].
///
/// Note: `win_filetime_from_systemtime` is implemented as a macro because this crate is
/// `[no_std]`. Implementing this via a function would require this crate to reference
/// `std::time::SystemTimeError`.
#[macro_export]
macro_rules! win_filetime_from_systemtime {
    // Keep in sync with tracelogging::win_filetime_from_systemtime.
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

extern crate alloc;
mod builder;
mod provider;
