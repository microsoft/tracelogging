[![Build Status](https://dev.azure.com/ms/TraceLogging/_apis/build/status/microsoft.TraceLogging?branchName=master)](https://dev.azure.com/ms/TraceLogging/_build/latest?definitionId=147&branchName=master)

# TraceLogging

TraceLogging is a set of technologies for emitting structured events. It is
primarily used with Event Tracing for Windows (ETW). Some parts of
TraceLogging are also available on other operating systems.

## TraceLogging for ETW (Windows)

On Windows, TraceLogging is a system for creating self-describing ETW events
that can be decoded without a manifest. The technology includes:

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

## TraceLogging for LTTng (Linux)

[TraceLogging for LTTng](LTTng/README.md) provides an LTTng Helpers library and a
TraceLoggingProvider.h header. This allows C/C++ developers to use the same
TraceLoggingProvider syntax to generate events for both ETW and LTTng.

## Reporting Security Issues

Security issues and bugs should be reported privately, via email, to the
Microsoft Security Response Center (MSRC) at <[secure@microsoft.com](mailto:secure@microsoft.com)>.
You should receive a response within 24 hours. If for some reason you do not, please follow up via
email to ensure we received your original message. Further information, including the
[MSRC PGP](https://technet.microsoft.com/en-us/security/dn606155) key, can be found in the
[Security TechCenter](https://technet.microsoft.com/en-us/security/default).

## Code of Conduct

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).

For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Contributing

Want to contribute? The team encourages community feedback and contributions. Please follow our [contributing guidelines](../CONTRIBUTING.md).

We also welcome [issues submitted on GitHub](https://github.com/Microsoft/TraceLogging/issues).

## Project Status

This project is currently in active development.

## Contact

The easiest way to contact us is via the [Issues](https://github.com/microsoft/TraceLogging/issues) page.
