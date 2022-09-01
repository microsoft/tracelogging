//! ## TraceLoggingDynamic for Rust
//!
//! tracelogging_dynamic supports writing TraceLogging (manifest-free) ETW events with
//! a runtime-defined schema. The events can be generated and collected on Windows Vista
//! or later. The events can be decoded on Windows 10 or later.
//!
//! This "dynamic" implementation of TraceLogging supports more functionality than the
//! implementation in TraceLoggingProvider.h (e.g. runtime-defined schema and arrays of
//! strings), but it also has higher runtime costs since the event schema is constructed
//! at runtime and the event fields are buffered (TraceLoggingProvider.h's compile-time
//! schema requirements make it possible to avoid buffering). This implementation is
//! intended for use only when the set of events is not known at compile-time. For example,
//! tracelogging_dynamic might be used to implement a middle-layer library providing
//! manifest-free ETW to a scripting language like JavaScript or Perl.
//!
//! See the documentation of the Provider class for usage details and examples.

#![no_std]
#![warn(missing_docs)]
#![allow(clippy::needless_return)]

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

extern crate alloc;
mod builder;
mod provider;

#[cfg(test)]
mod tests;
