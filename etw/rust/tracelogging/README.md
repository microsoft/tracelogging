# TraceLogging for Rust

The `tracelogging` crate provides a simple and efficient system for
logging TraceLogging events when the event schema is known at compile time.

This is similar to the C/C++
[TraceLoggingProvider.h](https://docs.microsoft.com/windows/win32/api/traceloggingprovider/)
implementation in the Windows SDK.

```rust
use tracelogging as tlg;

// Define a static variable for the "MyCompany.MyComponent" provider.
// Note that provider variable is not pub so it is not visible outside the
// module. To share the variable with multiple modules, put the define_provider
// in the parent module, e.g. in lib.rs.
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

## Configuration

This crate supports the following configurable features:

- `etw`: Use
  [Windows ETW](https://docs.microsoft.com/windows/win32/etw/about-event-tracing) APIs to
  log events. If not enabled, all logging operations will be no-ops.
  **Enabled by default.**
- `kernel_mode`: Use kernel-mode ETW APIs (e.g. `EtwWriteTransfer`) instead of
  user-mode ETW APIs (e.g. `EventWriteTransfer`).
- `macros`: Re-export the `define_provider!` and `write_event!` macros from the
  `tracelogging_macros` crate. **Enabled by default.**

In addition, this crate will log events only if compiled for a Windows operating system.
If compiled for a non-Windows operating system, all logging operations will be no-ops.
