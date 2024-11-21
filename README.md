# TraceLogging

TraceLogging is a set of technologies for emitting structured events. It is
primarily used with Event Tracing for Windows (ETW). Some parts of
TraceLogging are also available on other operating systems.

## TraceLogging for ETW (Windows)

[TraceLogging for ETW](etw/README.md) provides helpers for using TraceLogging with
[Event Tracing for Windows (ETW)](https://docs.microsoft.com/windows/win32/etw/about-event-tracing).

## TraceLogging for LTTng (Linux)

[TraceLogging for LTTng](LTTng/README.md) provides an LTTng Helpers library and a
TraceLoggingProvider.h header. This allows C/C++ developers to use the same
TraceLoggingProvider syntax to generate events for both ETW and [LTTng](https://lttng.org/).

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
