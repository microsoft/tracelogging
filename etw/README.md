# TraceLogging for ETW (Windows)

On Windows, TraceLogging is a system for creating self-describing ETW events
that can be decoded without a manifest. Support for TraceLogging on Windows is
included with the Windows OS and the Windows SDK. The technology includes:

- Rules for how to pack the data into the event payload and for how to send
  the event to the ETW system.
- Operating system support for decoding events, including
  [TDH APIs](https://docs.microsoft.com/windows/win32/api/tdh/nf-tdh-tdhgeteventinformation).
- SDK tools for decoding events, including
  [tracefmt](https://docs.microsoft.com/windows-hardware/drivers/devtest/tracefmt).
- [TraceLoggingProvider.h](https://docs.microsoft.com/windows/win32/api/traceloggingprovider/)
  header in the Windows SDK supports generating TraceLogging-encoded events using
  C or C++ for kernel and user-mode code.
- [EventSource](https://docs.microsoft.com/dotnet/api/system.diagnostics.tracing.eventsource)
  class supports generating TraceLogging-encoded events (when using the `Write` method or
  when constructed using the
  [EtwSelfDescribingEventFormat](https://docs.microsoft.com/dotnet/api/system.diagnostics.tracing.eventsourcesettings)
  flag).
- [LoggingChannel](https://docs.microsoft.com/uwp/api/windows.foundation.diagnostics.loggingchannel)
  Windows Runtime class supports generating TraceLogging-encoded events.

This project provides additional support for TraceLogging developers:

- [TraceLoggingDynamic for C++](cpp/traceloggingdynamic/README.md) provides support for generating
  runtime-dynamic events from C++ code.
- [TraceLoggingDynamic for C#](cs/traceloggingdynamic/README.md) provides support for generating
  runtime-dynamic events from C# code.
- [TraceLoggingDynamic for Python](python/traceloggingdynamic/README.md) provides support for generating
  runtime-dynamic events from Python3 code.
- [TraceLogging for Rust](rust/README.md) provides support for generating events from Rust code.
