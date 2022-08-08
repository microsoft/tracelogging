# Trace Logging Dynamic

[This implementation](TraceLoggingDynamic.h) of manifest-free ETW supports
more functionality than the implementation in
[TraceLoggingProvider.h](https://docs.microsoft.com/windows/win32/tracelogging/trace-logging-about),
but it also has higher runtime costs. This implementation is intended for use only when
the set of events is not known at compile-time. For example,
TraceLoggingDynamic.h might be used to implement a library providing
manifest-free ETW to a scripting language like JavaScript or Perl.

This header is not optimized for direct use by developers adding events
to their code. There is no way to make an optimal solution that
works for all of the intended target users. Instead, this header
provides various pieces that you can build upon to create a user-friendly
implementation of manifest-free ETW tailored for a specific scenario.

## HIGH-LEVEL API

The high-level API provides an easy way to get up and running with
TraceLogging ETW events.

### Contents

- Class:    tld::Provider
- Class:    tld::Event
- Class:    tld::EventBuilder
- Enum:     tld::Type

### Basic usage

- Create a Provider object.
- Check `provider.IsEnabled(...)` so that you don't do the remaining steps
  if nobody is listening for your event.
- Create an `Event<std::vector<BYTE>>` object.
- Add fields definitions (metadata) and values (data) using methods on
  Event. (You are responsible for making sure that the metadata you add
  matches the data you add -- the Event object does not validate this.)
- Some methods on the Event object return EventBuilder objects, which are
  used to build the fields of nested structures. Note that Event inherits
  from EventBuilder, so if you write a function that accepts an
  EventBuilder&, it will also accept an Event&.
- Once you've added all data and metadata, call `event.Write()` to send the
  event to ETW.

## LOW-LEVEL API

The low-level API provides components that you can mix and match to build
your own solution when the high-level components don't precisely meet your
needs. For example, you might use the high-level Provider class to manage
your REGHANDLE and the ETW callbacks, but you might use
EventMetadataBuilder and EventDataBuilder directly instead of using Event
so that you can cache event definitions.

### Contents

- Function: tld::RegisterProvider
- Function: tld::UnregisterProvider
- Function: tld::GetGuidForName
- Function: tld::SetInformation
- Class:    tld::ProviderMetadataBuilder
- Class:    tld::EventMetadataBuilder
- Class:    tld::EventDataBuilder
- Struct:   tld::EventDescriptor
- Class:    tld::ByteArrayWrapper
- Enum:     tld::InType
- Enum:     tld::OutType
- Enum:     tld::ProviderTraitType
- Function: tld::WriteEvent
- Function: tld::WriteEventEx
- Function: tld::PushBackAsUtf8
- Function: tld::MakeType
- Macro:    TLD_HAVE_EVENT_SET_INFORMATION
- Macro:    TLD_HAVE_EVENT_WRITE_EX

### Notes

- EventDataDescCreate is a native ETW API from `<evntprov.h>`. It is not
  part of this API, but you may want to use it to initialize
  EVENT_DATA_DESCRIPTOR structures instead of doing it directly.
- If you directly initialize your EVENT_DATA_DESCRIPTOR structures instead
  of using EventDataDescCreate, be sure to properly initialize the
  EVENT_DATA_DESCRIPTOR.Reserved field (e.g. by setting it to 0).
  Initializing the Reserved field is NOT optional. (EventDataDescCreate
  does correctly initialize the Reserved field.)
- When the API asks for a byte-buffer, you can use `std::vector<BYTE>` or
  another similar type. If you don't want to use a vector, the provided
  ByteArrayWrapper type allows you to use your own allocation strategy for
  the buffer.
- By default, TraceLogging events have Id=0 and Version=0, indicating
  that the event does not have an assigned Id. However, events can have
  Id and Version assigned (typically assigned manually). If you don't want
  to manage event IDs, set both Id and Version to 0. If you do assign
  IDs to your events, the Id must be non-zero and there should be a
  one-to-one mapping between {provider.Id + event.Id + event.Version} and
  {event metadata}. In other words, any event with a given non-zero
  Id+Version combination must always have exactly the same event metadata.
  (Note to decoders: this can be used as an optimization, but do not rely
  on providers to follow this rule.)
- TraceLogging events default to channel 11 (WINEVENT_CHANNEL_TRACELOGGING).
  This channel has no effect other than to mark the event as TraceLogging-
  compatible. Other channels can be used, but channels other than 11 will
  only work if the provider is running on a version of Windows that
  understands TraceLogging (Windows 7sp1 with latest updates, Windows 8.1
  with latest updates, Windows 10, or later). If your provider is running
  on a version of Windows that does not understand TraceLogging and you use
  a channel other than 11, the resulting events will not decode correctly.

### Low-level provider management

- Use tld::ProviderMetadataBuilder to build a provider metadata blob.
- If you don't have a specific provider GUID already selected, use
  tld::GetGuidForName to compute your provider GUID.
- Use tld::RegisterProvider to open the REGHANDLE.
- Use the REGHANDLE in calls to tld::WriteEvent.
- Use tld::UnregisterProvider to close the REGHANDLE.

### Low-level event management

- Use tld::EventMetadataBuilder to build an event metadata blob.
  - Use tld::Type values to define field types, or create non-standard
    types using tld::MakeType, tld::InType, and tld::OutType.
- Use tld::EventDataBuilder to build an event data blob.
- Create an EVENT_DESCRIPTOR for your event.
  - Optionally use the tld::EventDescriptor wrapper class to simplify
    initialization of EVENT_DESCRIPTOR structures.
- Allocate an array of EVENT_DATA_DESCRIPTORs. Size should be two more
  than you need for your event payload (the first two are reserved for the
  provider and event metadata).
- Use EventDataDescCreate to fill in the data descriptor array (skipping
  the first two) to reference your event payload data.
- Use tld::WriteEvent to write your event.
