# TraceLogging Dynamic for Rust

The `tracelogging_dynamic` crate provides a way to log TraceLogging events when
the event schema is not known at compile-time.

This implementation is less user-friendly and has higher runtime costs than the
implementation in the `tracelogging` crate. This implementation should only be used
when the set of events to log cannot be deteremined ahead of time. For example, this
might be useful when implementing a middle-layer library that provides generic logging
facilities to a dynamic upper layer.

```rust
use tracelogging_dynamic as tld;

// Pinning is required because the register() method sets up a callback with ETW.
let provider =
    Box::pin(tld::Provider::new("MyCompany.MyComponent", &tld::Provider::options()));

// Register the provider. If you don't register (or if register fails) then enabled()
// will always return false and write() will be a no-op.
unsafe {
    provider.as_ref().register();
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

## Configuration

This crate supports the following configurable features:

- `etw`: Use
  [Windows ETW](https://docs.microsoft.com/windows/win32/etw/about-event-tracing) APIs to
  log events. If not enabled, all logging operations will be no-ops.
  **Enabled by default.**

In addition, this crate will log events only if compiled for a Windows operating system.
If compiled for a non-Windows operating system, all logging operations will be no-ops.
