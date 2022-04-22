# traceloggingdynamic

The **traceloggingdynamic** module provides support for generating
[TraceLogging](https://docs.microsoft.com/windows/win32/tracelogging/trace-logging-portal)-encoded
[ETW (Event Tracing for Windows)](https://docs.microsoft.com/windows/win32/etw/event-tracing-portal)
events directly from Python (no C/C++ code). The events can be generated and
collected on Windows Vista or later. The events can be decoded on Windows 10
or later.

This is a middle-layer API:

- Directly exposes ETW concepts rather than wrapping them in Python idioms.
- Makes it possible for the caller to be efficient even when that means the
  caller might need to do more work (e.g. take names as bytes so that the
  caller has the option of avoiding the string encoding for each event).
- Could be used directly by a consumer or could be wrapped in a higher-level
  module to create a more Python-friendly API.

## Public

- `Provider` class: represents an ETW data source with a name and id. Core
  methods: provider.is_enabled(event_level, event_keyword),
  provider.write(event_builder).
- `EventBuilder` class: Used to put together the data for an event. Core
  methods: `eb.reset(event_name, event_level, event_keyword, [more...])`,
  `eb.add_FIELDTYPE(field_name, field_value, [out_type, tag])`. When the event
  is ready, send it to ETW with `provider.write(eb)`.
- `OutType` enum: Formatting hints that can be added to a field.
- `providerid_from_name` function: Hashes a provider name string to generate
  a provider id UUID. Hash is generated using the same algorithm as other ETW
  APIs and tools such as .NET EventSource and WinRT LoggingChannel.

## Example

```python
# At start of program:
provider = Provider(b'MyCompany.MyGroup.MyComponent')

# When you want to generate an event:
my_field_value = True
my_event_level = 5      # 5 = Verbose
my_event_keyword = 0x21 # User-defined bits indicating categories.
if provider.is_enabled(my_event_level, my_event_keyword): # Anybody listening?
    eb = EventBuilder()
    eb.reset(b'MyEventName', my_event_level, my_event_keyword)
    eb.add_bool32(b'MyFieldName', my_field_value) # as needed to add fields.
    provider.write(eb)
```

## Notes

Collect the events using Windows SDK tools like traceview or tracelog.
Decode the events using Windows SDK tools like traceview or tracefmt.
For example, for `Provider(b'MyCompany.MyComponent')`:

```powershell
tracelog -start MyTrace -f MyTraceFile.etl -guid *MyCompany.MyComponent -level 5 -matchanykw 0xf
<run your Python program>
tracelog -stop MyTrace
tracefmt -o MyTraceData.txt MyTraceFile.etl
```

ETW events are limited in size (event size = headers + metadata + data).
Windows will drop any event that is larger than 64KB and will drop any event
that is larger than the buffer size of the recording session. To help prevent
unexpected event loss, this module will raise an exception if the metadata
exceeds 32KB or if the data exceeds 64KB. To help track down event loss, this
module will assert if it detects that ETW dropped an event due to size.

Most ETW decoding tools are unable to decode an event with more than 128
fields. This module will raise an exception if you add more than 128 fields to
an event. Note that sequence fields and binary fields each count as 2 fields.
