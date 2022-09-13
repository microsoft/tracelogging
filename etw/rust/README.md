# TraceLogging for Rust

## tracelogging

The [`tracelogging`](tracelogging) crate provides a simple and efficient system for
logging TraceLogging events when the event schema is known at compile time.

This is similar to the C/C++
[TraceLoggingProvider.h](https://docs.microsoft.com/windows/win32/api/traceloggingprovider/)
implementation in the Windows SDK.

```rust
use tracelogging as tlg;

// Define a static variable for the "MyCompany.MyComponent" provider.
tlg::define_provider!(
    MY_PROVIDER,              // The static symbol to use for this provider.
    "MyCompany.MyComponent"); // The provider's name (string literal).

// Register the provider at module initialization. If you don't register (or if
// register fails) then MY_PROVIDER.enabled() will always return false, the
// write_event macro will be a no-op, and MY_PROVIDER.unregister() will be a no-op.
// Safety: MUST call MY_PROVIDER.unregister() before module unload.
unsafe { MY_PROVIDER.register(); }

// As necessary, call write_event to send events to ETW.
let field1_value = "String Value";
let field2_value = 42u32;
tlg::write_event!(
    MY_PROVIDER,                    // The provider to use for the event.
    "MyEventName",                  // The event's name (string literal).
    level(Warning),                 // Event's severity level.
    keyword(0x23),                  // Event category bits.
    str8("Field1", field1_value),   // Add a string field to the event.
    u32("Field2", &field2_value),   // Add an integer field to the event.
);

// Before module unload, unregister the provider.
MY_PROVIDER.unregister();
```

The `tracelogging` crate depends on the
[`tracelogging_macros`](tracelogging_macros) crate which implements the
`define_provider!` and `write_event!` macros.

## tracelogging_dynamic

The [`tracelogging_dynamic`](tracelogging_dynamic) crate provides a way to log
TraceLogging events when the event schema is not known at compile-time.

This implementation is less user-friendly and has higher runtime costs than the
implementation in the `tracelogging` crate. This implementation should only be used
when the set of events to log cannot be deteremined ahead of time. For example, this
might be useful when implementing a middle-layer library that provides generic logging
facilities to a dynamic upper layer.

```rust
use tracelogging_dynamic as tld;

// Pinning is required because the register() method sets up a callback with ETW.
let mut provider = Box::pin(tld::Provider::new());

// Register the provider with name "MyCompany.MyComponent". If you don't register (or
// if register fails) then enabled() will always return false and write() will be a
// no-op.
unsafe {
    provider.as_mut().register("MyCompany.MyComponent", &tld::Provider::options());
}

// If provider is not enabled for a given level + keyword, the write() call will do
// nothing. Check enabled(level, keyword) before building the event so we don't waste
// time on an event that nobody will receive.
let my_event_level = tld::Level::Verbose; // Severity level.
let my_event_keyword = 0x123; // User-defined category bits.
if provider.enabled(my_event_level, my_event_keyword) {
    let field1_value = "FieldValue";
    let field2_value = b'A';
    // Create and write an event with two fields:
    tld::EventBuilder::new()
        // Most events specify 0 for event tag.
        .reset("MyEventName", my_event_level, my_event_keyword, 0)
        // Most fields use Default for event format and 0 for field tag.
        .add_str8("FieldName1", field1_value, tld::OutType::Default, 0)
        .add_u8("FieldName2", field2_value, tld::OutType::String, 0)
        // If activity_id is None, event uses the current thread's activity.
        // If related_id is None, event will not have a related activity.
        .write(&provider, None, None);
}
```

The `tracelogging_dynamic` crate depends on the
[`tracelogging`](tracelogging) crate which implements underlying API support.
