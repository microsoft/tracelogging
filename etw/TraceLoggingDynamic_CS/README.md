# Trace Logging Dynamic

[This implementation](TraceLoggingDynamic.cs) of manifest-free ETW supports
more functionality than the implementation in
[System.Diagnostics.Tracing.EventSource](https://docs.microsoft.com/dotnet/api/system.diagnostics.tracing.eventsource),
but it also has higher runtime costs. This implementation is intended for use only when
the set of events is not known at compile-time. For example,
TraceLoggingDynamic.cs might be used to implement a library providing
manifest-free ETW to a higher-level API that does not enforce compile-time
event layout.

```cs
// At start of program:
static readonly EventProvider p = new EventProvider("MyProviderName");

// When you want to write an event:
if (p.IsEnabled(EventLevel.Verbose, 0x1234)) // Anybody listening?
{
    var eb = new EventBuilder(); // Or reuse an existing EventBuilder
    eb.Reset("MyEventName", EventLevel.Verbose, 0x1234);
    eb.AddInt32("MyInt32FieldName", intValue);
    eb.AddUnicodeString("MyStringFieldName", stringValue);
    p.Write(eb);
}
```

The EventProvider class encapsulates an ETW REGHANDLE, which is a handle
through which events for a particular provider can be written. The
EventProvider instance should generally be created at component
initialization and one instance should be shared by all code within the
component that needs to write events for the provider. In some cases, it
might be used in a smaller scope, in which case it should be closed via
Dispose().

The EventBuilder class is used to build an event. It stores the event name,
field names, field types, and field values. When all of the fields have
been added to the builder, call eventProvider.Write(eventBuilder, ...) to
send the eventBuilder's event to ETW.

To reduce performance impact, you might want to skip building the event if
there are no ETW listeners. To do this, use eventProvider.IsEnabled(...).
It returns true if there are one or more ETW listeners that would be
interested in an event with the specified level and keywords. You only
need to build and write the event if IsEnabled returned true for the level
and keyword that you will use in the event.

The EventBuilder object is a class, and it stores two byte[] buffers.
This means each new EventBuilder object generates garbage. You can
minimize garbage by reusing the same EventBuilder object for multiple
events instead of creating a new EventBuilder for each event.

## Notes

Collect the events using Windows SDK tools like traceview or tracelog.
Decode the events using Windows SDK tools like traceview or tracefmt.
For example, for `EventProvider("MyCompany.MyComponent")`:

```powershell
tracelog -start MyTrace -f MyTraceFile.etl -guid *MyCompany.MyComponent -level 5 -matchanykw 0xf
<run your program>
tracelog -stop MyTrace
tracefmt -o MyTraceData.txt MyTraceFile.etl
```

ETW events are limited in size (event size = headers + metadata + data).
Windows will drop any event that is larger than 64KB and will drop any event
that is larger than the buffer size of the recording session.

Most ETW decoding tools are unable to decode an event with more than 128
fields.
