// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/*
This is an implementation of TraceLoggingProvider.h that writes data to LTTNG.

Quick start:

#include <tracelogging/TraceLoggingProvider.h>

TRACELOGGING_DEFINE_PROVIDER( // defines MyProvider
    MyProvider, // Name of the provider symbol
    "MyCompany.MyComponent.MyProvider", // Human-readable provider name
    // {d5b90669-1aad-5db8-16c9-6286a7fcfe33} // Provider guid (ignored by LTTNG)
    (0xd5b90669,0x1aad,0x5db8,0x16,0xc9,0x62,0x86,0xa7,0xfc,0xfe,0x33));

int main(int argc, char* argv[])
{
    TraceLoggingRegister(MyProvider);
    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingString(argv[0], "arg0"), // field name is "arg0"
        TraceLoggingInt32(argc)); // field name is implicitly "argc"
    TraceLoggingUnregister(MyProvider);
    return 0;
}

Usage note:

Symbols starting with "TRACELOGGING" or "TraceLogging" are part of the public
interface of this header. Symbols starting with "_tlg" are private internal
implementation details that are not supported for use outside this header and
may change in future versions of this header.

TraceLoggingProvider.h for LTTNG behaves differently from the ETW version:

- LTTNG does not support multiple providers with the same name in a process.
  If two providers with the same name try to register in the same process, the
  second provider's registration will fail.
- LTTNG does not support multiple events with the same name in a provider
  (works at runtime but decoding will fail).
- LTTNG does not support multiple fields with the same name in an event (works
  at runtime but decoding will fail).
- LTTNG does not support the use of double-quote characters in provider or
  event names (works at runtime but decoding will fail).
- LTTNG does not support the use of non-identifier characters in field names
  (works at runtime but decoding will fail). Field names may only use letters,
  numbers, and underscores.
- The total length of the provider name and the event name (including keyword
  suffix) must be less than 255.
- No support for TraceLoggingProviderEnabled. Instead, use
  TraceLoggingEventEnabled (new LTTNG-only function) when working with LTTNG.
- No support for TraceLoggingHProvider. In the ETW implementation of
  TraceLoggingProvider.h, provider symbols are handles (pointers), and can be
  passed around and stored as variables (though we�ve always recommended
  against doing this since it breaks static analysis). In the LTTNG
  implementation of TraceLoggingProvider.h, provider symbols are tokens, not
  handles or pointers. You cannot store a provider symbol in a variable, pass
  it as a parameter or return it from a function. The provider symbol used as
  the first parameter to TraceLoggingWrite must be the same symbol as was used
  in TRACELOGGING_DEFINE_PROVIDER.
  - As a consequence of this, wil\TraceLogging.h will not work (no way to
    implement TRACELOGGING_DEFINE_PROVIDER_STORAGE).
- Limited support for TraceLoggingLevel.
  - You must use LTTNG level values, not ETW level values. For example,
    TraceLoggingLevel(4) means level="info" under ETW, but is level="warning"
    under LTTNG.
  - In TraceLoggingProvider.h for LTTNG, the default level is "debug" (14).
    In TraceLoggingProvider.h for ETW, the default level is "verbose" (5).
- Limited support for TraceLoggingKeyword. TraceLoggingKeyword adds a suffix
  to the event name for each enabled keyword so that you can filter events
  based on the suffix, e.g. keyword 0x5 will turn into suffix ";k0;k2;".
- Limited support for TraceLoggingWriteActivity. TraceLoggingWriteActivity
  adds "_ms_ActivityId" and "_ms_RelatedActivityId" fields to the event.
- Limited support for TraceLoggingOpcode. TraceLoggingOpcode adds an
  "_ms_Opcode" field to the event.
- Limited support for TraceLoggingChannel. TraceLoggingChannel adds an
  "_ms_Channel" field to the event.
- Limited support for TraceLoggingEventTag. TraceLoggingEventTag adds an
  "_ms_EventTag" field to the event.
- Limited support for type formatting.
  - TraceLoggingString, TraceLoggingCountedString, and TraceLoggingChar use
    UTF-8 encoding. (For ETW, they would use the decoding system's default
    ANSI code page.)
  - TraceLoggingWinError, TraceLoggingNTStatus, and TraceLoggingHResult will
    be formatted as integer, not as message.
  - TraceLoggingGuid will accept any value V, and will format (uint8*)&V as an
    array of 16 HexInt8 values.
  - TraceLoggingSystemTime will accept any value V, and will format
    (uint16*)&V as an array of 8 UInt16 values.
  - TraceLoggingFileTime will accept any value V, and will format *(uint64*)&V
    as a single UInt64 value.
  - TraceLoggingBinary, TraceLoggingSocketAddress, and TraceLoggingSid will
    format the data as an array of HexInt8 values.
- Field descriptions and field tags will be ignored.
- TraceLoggingDescription will be ignored.
- TraceLoggingCustomAttribute will be ignored.
- TraceLoggingOptionGroup will be ignored.
- TraceLoggingOptionMicrosoftTelemetry will be ignored.
- TraceLoggingStruct will be ignored.
- TraceLoggingValue will not accept GUID, FILETIME, SYSTEMTIME, or SID values
  (primarily because there is no consistent definition for the corresponding
  structures on Linux).
- No support for TraceLoggingProviderId. LTTNG does not identify providers by
  GUID. Instead, use TraceLoggingProviderName (new LTTNG-only function).
- No support for TraceLoggingRegisterEx. LTTNG does not support callbacks.
- No support for TraceLoggingSetInformation.
- No support for TraceLoggingWriteEx.
- No support for TraceLoggingPacked macros.
- No support for TraceLoggingFloat*Array macros.
- No support for TraceLoggingAnsiString, TraceLoggingUnicodeString (i.e. the
  NT kernel ANSI_STRING and UNICODE_STRING structures). Could be added.
- Characters and strings of char16_t, char32_t, or wchar_t will be transcoded
  to UTF-8 before being written to the trace. To support non-ASCII data, a
  char16/char32/wchar will transcode into a variable-length UTF-8 string.
- wchar_t is a 32-bit type on Linux. Use "Char16" and "String16" macros (based
  on char16_t) if you need UTF-16.
- Added support for logging characters and strings based on char16_t and
  char32_t types.

Future improvements to LTTNG may allow TraceLoggingProvider.h to resolve the
following issues:
- Support for better handling of invalid provider, event, or field names.
- Support for multiple instances of a given provider name in a process.
- Support for multiple different events with the same name in a provider.
- Support for TraceLoggingStruct.
- Support for TraceLoggingFloat*Array macros (i.e. support for
  array/sequence of floats).
- Support for keywords.
- Support for better formatting of GUID and timestamp fields.
- Support for better handling of activity IDs.
- Support for traditional ETW metadata such as provider id, opcode, channel,
  event tag, field tag.

Open questions:
- Any better way to encode GUID in the trace to make it decode more nicely?
  Currently, GUID logs as array-of-HexInt8, which looks awful when decoded.
  Once LTTNG adds support for struct, we can do better, but until then should
  we do something special such as converting them to string before logging?
  (Note: not practical to convert array-of-guid to a string.)
- Is there a GUID struct definition we should be using? Note that uint8_t[16]
  is not specific enough to be used with TraceLoggingValue -- we need a struct
  of some kind.
- Should there be more-specific type checking for FILETIME, SYSTEMTIME, SID?
  Currently we just accept anything.
- Do we even care about FILETIME, SYSTEMTIME, SID on Linux?
- Are there important Linux types that deserve first-class support? (Probably
  not worth worrying about until LTTNG adds support for struct.)
*/

#pragma once
#ifndef _TRACELOGGINGPROVIDER_
#define _TRACELOGGINGPROVIDER_

#include <lttngh/LttngHelpers.h>
#include <assert.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C"
{
#endif // __cplusplus

#ifdef __EDG__
#pragma region Internal helper macros
#endif

#ifndef _tlg_NOEXCEPT
#ifndef __cplusplus
#define _tlg_NOEXCEPT
#else
#define _tlg_NOEXCEPT noexcept
#endif // __cplusplus
#endif // _tlg_NOEXCEPT

#ifndef _tlg_WEAK_ATTRIBUTES
#define _tlg_WEAK_ATTRIBUTES __attribute__((weak, visibility("hidden"))) lttng_ust_notrace
#endif // _tlg_WEAK_ATTRIBUTES

#ifndef _tlg_INLINE_ATTRIBUTES
#define _tlg_INLINE_ATTRIBUTES lttng_ust_notrace
#endif // _tlg_INLINE_ATTRIBUTES

#ifndef __cplusplus
#define _tlg_EXTERN_C extern
#else
#define _tlg_EXTERN_C extern "C"
#endif // __cplusplus

#if (__WCHAR_MAX == 0x7fffffff) || (__WCHAR_MAX == 0xffffffff)
#define _tlgWcharNN 32
#define _tlgWcharNN_type char32_t
#elif (__WCHAR_MAX == 0x7fff) || (__WCHAR_MAX == 0xffff)
#define _tlgWcharNN 16
#define _tlgWcharNN_type char16_t
#else
#error Unsupported wchar_t type.
#endif // __WCHAR_MAX

/*
Do not use these directly. Use wrapper macros such as TraceLoggingInt32.
These handler macros may be renamed or removed in future versions of this header.
*/
#define _tlg_ArgIgnored() \
    (_tlg_Ignored)
#define _tlg_ArgLevel(eventLevel) \
    (_tlg_Level, eventLevel)
#define _tlg_ArgKeyword(eventKeyword) \
    (_tlg_Keyword, eventKeyword)
#define _tlg_ArgInt(ctype, value, intFieldType, isSigned, ndt) \
    (_tlg_Int, ctype, value, intFieldType, isSigned, ndt)
#define _tlg_ArgEnum(ctype, value, intFieldType, isSigned, ndt, etype) \
    (_tlg_Enum, ctype, value, etype, intFieldType, isSigned, ndt) // Note: parameter reordered for aesthetic reasons.
#define _tlg_ArgIntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt) \
    (_tlg_IntArrayByRef, ctype, value, cValues, intFieldType, isSigned, ndt)
#define _tlg_ArgIntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) \
    (_tlg_IntArray, ctype, pValues, cValues, intFieldType, isSigned, ndt)
#define _tlg_ArgIntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) \
    (_tlg_IntFixedArray, ctype, pValues, cValues, intFieldType, isSigned, ndt)
#define _tlg_ArgFloat(ctype, value, ndt) \
    (_tlg_Float, ctype, value, ndt)
#define _tlg_ArgFloatArray(ctype, pValues, cValues, ndt) \
    (_tlg_FloatArray, ctype, pValues, cValues, ndt)
#define _tlg_ArgFloatFixedArray(ctype, pValues, cValues, ndt) \
    (_tlg_FloatFixedArray, ctype, pValues, cValues, ndt)
#define _tlg_ArgChar8(ctype, value, ndt) \
    (_tlg_Char8, ctype, value, ndt)
#define _tlg_ArgCharNN(ctype, value, NN, ndt) \
    (_tlg_CharNN, ctype, value, NN, ndt)
#define _tlg_ArgString8(ctype, pszValue, ndt) \
    (_tlg_String8, ctype, pszValue, ndt)
#define _tlg_ArgStringNN(ctype, pszValue, NN, ndt) \
    (_tlg_StringNN, ctype, pszValue, NN, ndt)
#define _tlg_ArgCountedString8(ctype, pchValue, cchValue, ndt) \
    (_tlg_CountedString8, ctype, pchValue, cchValue, ndt)
#define _tlg_ArgCountedStringNN(ctype, pchValue, cchValue, NN, ndt) \
    (_tlg_CountedStringNN, ctype, pchValue, cchValue, NN, ndt)
#define _tlg_ArgBinary(ctype, pValue, cbValue, ndt) \
    (_tlg_Binary, ctype, pValue, cbValue, ndt)
#define _tlg_ArgSid(ctype, pValue, ndt) \
    (_tlg_Sid, ctype, pValue, ndt)
#define _tlg_ArgGuidPtr(ctype, pValue, name) \
    (_tlg_GuidPtr, ctype, pValue, name) // Accepts NULL. Supports RelatedActivityId.
#define _tlg_ArgBuffer(ctype, pValue, ndt) \
    (_tlg_Buffer, ctype, pValue, ndt)

#ifdef __EDG__
#pragma endregion
#endif

#ifdef __EDG__
#pragma region Public interface
#endif

    /*
This structure is left undefined to ensure a compile error for any attempt to
copy or dereference the provider symbol. The provider symbol is a token, not a
variable or a handle.
*/
    struct TraceLoggingProviderSymbol;

/*
Macro TRACELOGGING_DECLARE_PROVIDER(providerSymbol):
Invoke this macro to forward-declare a provider symbol that will be defined
elsewhere using TRACELOGGING_DEFINE_PROVIDER. TRACELOGGING_DECLARE_PROVIDER
would typically be used in a header.

An invocation of

    TRACELOGGING_DECLARE_PROVIDER(MyProvider);

can be thought of as expanding to something like this:

    extern "C" TraceLoggingProviderSymbol MyProvider;

A symbol declared by TRACELOGGING_DECLARE_PROVIDER must later be defined in a
.c or .cpp file using the TRACELOGGING_DEFINE_PROVIDER macro.
*/
#define TRACELOGGING_DECLARE_PROVIDER(providerSymbol)                                                                                                                 \
    _tlg_EXTERN_C struct lttng_ust_tracepoint *__start___tracepoints_ptrs_##providerSymbol[] __attribute__((weak, visibility("hidden")));                             \
    _tlg_EXTERN_C struct lttng_ust_tracepoint *__stop___tracepoints_ptrs_##providerSymbol[] __attribute__((weak, visibility("hidden")));                              \
    _tlg_EXTERN_C struct lttng_event_desc const *__start___eventdesc_ptrs_##providerSymbol[] __attribute__((weak, visibility("hidden")));                             \
    _tlg_EXTERN_C struct lttng_event_desc const *__stop___eventdesc_ptrs_##providerSymbol[] __attribute__((weak, visibility("hidden")));                              \
    _tlg_EXTERN_C struct TraceLoggingProviderSymbol providerSymbol __attribute__((visibility("hidden"))); /* Empty provider variable to help with code navigation. */ \
    _tlg_EXTERN_C struct _tlg_Provider_t _tlgProv_##providerSymbol __attribute__((visibility("hidden")))  /* Actual provider variable is hidden behind prefix. */

/*
Macro TRACELOGGING_DEFINE_PROVIDER(providerSymbol, "ProviderName", providerId, [option]):
Invoke this macro to create the global storage for a provider. The provider
name must be a string literal (not a variable) and must not contain any '"' or
'\0' characters.

An invocation of

    TRACELOGGING_DEFINE_PROVIDER(MyProvider, "MyProviderName",
        (0xb3864c38, 0x4273, 0x58c5, 0x54, 0x5b, 0x8b, 0x36, 0x08, 0x34, 0x34, 0x71));

can be thought of as expanding to something like this:

    extern "C" TraceLoggingProvider MyProvider = { ... };

The providerId specifies a unique GUID that identifies the provider. The
providerId parameter must be a parenthesized list of 11 integers e.g.
(n1, n2, n3, ... n11).

After the providerId GUID, you may optionally specify a TraceLoggingOption...
macro to configure your provider, e.g.

    TRACELOGGING_DEFINE_PROVIDER(MyProvider, "MyProviderName",
        (0xb3864c38, 0x4273, 0x58c5, 0x54, 0x5b, 0x8b, 0x36, 0x08, 0x34, 0x34, 0x71),
        TraceLoggingOptionGroup(0xfaaf2f61, 0x9b26, 0x4591, 0x9b, 0xb1, 0xb9, 0xb8, 0xba, 0xe2, 0xd3, 0x4c));

Note that the provider handle is created in the "unregistered" state. A call
to TraceLoggingWrite with an unregistered handle is a no-op. Call
TraceLoggingRegister to register the handle.

LTTNG-specific:
- Total length of providerName + eventName must be less than 253.
- The providerName must not contain any double-quote characters.
- Only one provider with a given name may be registered per process.
- The providerId is currently ignored.
*/
#define TRACELOGGING_DEFINE_PROVIDER(providerSymbol, providerName, providerId, ...)                 \
    TRACELOGGING_DECLARE_PROVIDER(providerSymbol);                                                  \
    static_assert(sizeof("" providerName) <= LTTNG_UST_SYM_NAME_LEN - 3,                            \
                  "TRACELOGGING_DEFINE_PROVIDER providerName must be no more than 253 characters"); \
    _tlgParseProviderId(providerId) struct _tlg_Provider_t _tlgProv_##providerSymbol = {{("" providerName), NULL, 0, {}, {}, 0, LTTNG_UST_PROVIDER_MAJOR, LTTNG_UST_PROVIDER_MINOR, {}}, 0}

/*
Macro TraceLoggingOptionGroup(g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11):
Wrapper macro for use in TRACELOGGING_DEFINE_PROVIDER that declares the
provider's membership in a provider group.

LTTNG semantics: TraceLoggingOptionGroup has no effect.
*/
#define TraceLoggingOptionGroup(g1, g2, g3, g4, g5, g6, g7, g8, g9, g10, g11) \
    _tlg_ArgIgnored()

/*
Macro TraceLoggingUnregister(providerSymbol):
Call this function to unregister your provider. Normally you will register at
component initialization and unregister at component shutdown.

Note that unregistration is important, especially in the case of a shared
object that might be dynamically unloaded before the process ends.

Thread safety: It is not safe to overlap calls to TraceLoggingRegister and
TraceLoggingUnregister. In other words, you must not call TraceLoggingRegister
or TraceLoggingUnregister while another call to TraceLoggingRegister or
TraceLoggingUnregister might be in progress. In addition, you must not call
TraceLoggingRegister on a handle that is already registered.

It is safe to call TraceLoggingUnregister on a handle that has not been
registered (e.g. if the call to TraceLoggingRegister failed).

After unregistering a provider, it is ok to register it again. In other words,
the following is ok:

    TRACELOGGING_DECLARE_PROVIDER(hProvider);
    ...
    TraceLoggingRegister(hProvider);
    ...
    TraceLoggingUnregister(hProvider);
    ...
    TraceLoggingRegister(hProvider);
    ...
    TraceLoggingUnregister(hProvider);

However, re-registering a provider should only happen because a component
has been uninitialized and then reinitialized. You should not register and
unregister a provider each time you need to write a few events.
*/
#define TraceLoggingUnregister(providerSymbol) _tlg_ProviderUnregister( \
    &_tlgProv_##providerSymbol, __start___tracepoints_ptrs_##providerSymbol)

/*
Macro TraceLoggingRegister(providerSymbol):
Call this function to register your provider with LTTNG.

The provider handle must be in the "unregistered" state.

Returns 0 to indicate success.

Refer to the documentation in TraceLoggingUnregister for additional information
about registration.

Note that it is ok to ignore failure - if TraceLoggingRegister fails,
TraceLoggingWrite and TraceLoggingUnregister will be no-ops.
*/
#define TraceLoggingRegister(providerSymbol) _tlg_ProviderRegister(                          \
    &_tlgProv_##providerSymbol,                                                              \
    __start___tracepoints_ptrs_##providerSymbol, __stop___tracepoints_ptrs_##providerSymbol, \
    __start___eventdesc_ptrs_##providerSymbol, __stop___eventdesc_ptrs_##providerSymbol)

/*
Macro TraceLoggingEventEnabled(providerSymbol, "EventName"):
Returns true (non-zero) if the specified event is enabled.
*/
#define TraceLoggingEventEnabled(providerSymbol, eventName)                                            \
    ({                                                                                                 \
        static int const *_tlg_pState;                                                                 \
        caa_likely(_tlg_pState)                                                                        \
            ? caa_unlikely(CMM_LOAD_SHARED(*_tlg_pState))                                              \
            : caa_unlikely(_tlg_EventEnabled(                                                          \
                  &_tlgProv_##providerSymbol,                                                          \
                  ("" eventName),                                                                      \
                  __start___eventdesc_ptrs_##providerSymbol, __stop___eventdesc_ptrs_##providerSymbol, \
                  &_tlg_pState));                                                                      \
    })

/*
Macro TraceLoggingProviderName(providerSymbol):
Returns the provider's name as a nul-terminated const char*.
*/
#define TraceLoggingProviderName(providerSymbol) \
    _tlg_ProviderName(&_tlgProv_##providerSymbol)

/*
Macro TraceLoggingWrite(providerSymbol, "EventName", args...):
Invoke this macro to log an event.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingInt32(myIntVar),
        TraceLoggingWideString(myString));

The eventName parameter must be a string literal (not a variable) and must not
contain any '"' or '\0' characters.

Supports up to 99 args (subject to compiler limitations). Each arg must be a
wrapper macro such as TraceLoggingLevel, TraceLoggingKeyword, TraceLoggingInt32,
TraceLoggingString, etc.

LTTNG-specific:
- Total length of providerName + eventName must be less than 253.
- The eventName must not contain any double-quote characters.
- LTTNG does not support multiple events with the same name in a provider.
  This will not be detected at compile-time or runtime, but will lead to
  problems during trace decoding.
*/
#define TraceLoggingWrite(providerSymbol, eventName, ...)                                  \
    _tlg_Write_imp(lttngh_EventProbe,                                                      \
                   providerSymbol, eventName,                                              \
                   _tlg_ArgGuidPtr(void, lttngh_ActivityIdFilter(NULL), "_ms_ActivityId"), \
                   ##__VA_ARGS__)

/*
Macro TraceLoggingWriteActivity(providerSymbol, "EventName", pActivityId, pRelatedActivityId, args...):
Invoke this macro to log an event with ActivityId and RelatedActivityId data.

Example:

    TraceLoggingWriteActivity(MyProvider, "MyEventName",
        pActivityGuid,
        pRelatedActivityGuid, // Usually NULL (non-NULL for opcode START).
        TraceLoggingOpcode(WINEVENT_OPCODE_START),
        TraceLoggingInt32(myIntVar),
        TraceLoggingWideString(myString));

The event name must be a string literal (not a variable) and must not contain
any '"' or '\0' characters.

Supports up to 99 args (subject to compiler limitations). Each arg must be a
wrapper macro such as TraceLoggingLevel, TraceLoggingKeyword, TraceLoggingInt32,
TraceLoggingString, etc.

LTTNG-specific:
- Total length of providerName + eventName must be less than 253.
- The eventName must not contain any double-quote characters.
- LTTNG does not support multiple events with the same name in a provider.
  This will not be detected at compile-time or runtime, but will lead to
  problems during trace decoding.
- TraceLoggingWriteActivity is the same as TraceLoggingWrite except that it
  allows manual control of "_ms_ActivityId" and "_ms_RelatedActivityId" fields.
*/
#define TraceLoggingWriteActivity(providerSymbol, eventName, pActivityId, pRelatedActivityId, ...) \
    _tlg_Write_imp(lttngh_EventProbe,                                                              \
                   providerSymbol, eventName,                                                      \
                   _tlg_ArgGuidPtr(void, lttngh_ActivityIdFilter(pActivityId), "_ms_ActivityId"),  \
                   _tlg_ArgGuidPtr(void, pRelatedActivityId, "_ms_RelatedActivityId"),             \
                   ##__VA_ARGS__)

/*
Macro TraceLoggingLevel(eventLevel)
Wrapper macro for setting the event's level.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingLevel(lttngh_Level_WARNING),
        TraceLoggingWideString(myString));

The eventLevel parameter must be a compile-time constant 0 to 255. If no
TraceLoggingLevel(n) arg is set on an event, the event will default to level
14 (DEBUG). If multiple TraceLoggingLevel(n) args are provided, the level
value from the last TraceLoggingLevel(n) will be used.

LTTNG-specific:
- The level values are different from those used by ETW.
- Constants for the LTTNG level values are provided in LttngHelpers.h.
- The default level is 14 (DEBUG).
*/
#define TraceLoggingLevel(eventLevel) _tlg_ArgLevel(eventLevel)

/*
Macro TraceLoggingKeyword(eventKeyword):
Wrapper macro for setting the event's keyword(s).

LTTNG semantics: Adds a suffix to the event name.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingKeyword(MyNetworkingKeyword),
        TraceLoggingWideString(myString));

The eventKeyword parameter must be a compile-time constant 0 to UINT64_MAX.
Each bit in the parameter corresponds to a user-defined event category. If an
event belongs to multiple categories, the bits for each category should be
OR'd together to create the event's keyword value. If no
TraceLoggingKeyword(n) arg is provided, the default keyword is 0. If multiple
TraceLoggingKeyword(n) args are provided, they are OR'd together.

LTTNG-specific:
- Keywords are not natively supported by LTTNG. They are simulated by adding a
  suffix to the event name to allow for pattern matching on the event name.
  For example, an event specified as "MyEventName" with keyword 0x5 (bits 0
  and 2 are set) would result in a final event name of "MyEventName;k0;k2;".
  That way, you can enable all events that belong to category 0x4 by enabling
  all events with names containing the string ";k2;".
*/
#define TraceLoggingKeyword(eventKeyword) _tlg_ArgKeyword(eventKeyword)

/*
Macro TraceLoggingOpcode(eventOpcode):
Wrapper macro for setting the event's opcode.

LTTNG semantics: Adds an "_ms_Opcode" field to the event.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingOpcode(WINEVENT_OPCODE_START),
        TraceLoggingWideString(myString));

The eventOpcode parameter must be a compile-time constant 0 to 255 (typically
a WINEVENT_OPCODE_??? constant from LttngHelpers.h). If multiple
TraceLoggingOpcode(n) args are provided, the value from the last
TraceLoggingOpcode(n) is used.
*/
#define TraceLoggingOpcode(eventOpcode) TraceLoggingHexUInt8(_tlg_ENSURE_CONST(Opcode, eventOpcode), "_ms_Opcode")

/*
Macro TraceLoggingChannel(eventChannel)
Wrapper macro for setting the event's channel. (Advanced scenarios.)

LTTNG semantics: Adds a "_ms_Channel" field to the event.
*/
#define TraceLoggingChannel(eventChannel) TraceLoggingUInt8(_tlg_ENSURE_CONST(Channel, eventChannel), "_ms_Channel")

/*
Macro TraceLoggingEventTag(eventTag):
Wrapper macro for setting the event's tag(s).

LTTNG semantics: Adds an "_ms_EventTag" field to the event.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingEventTag(0x200000),
        TraceLoggingWideString(myString));

Tags are a 28-bit user-defined metadata field. The semantics of the tags are
defined by the event consumer.
*/
#define TraceLoggingEventTag(eventTag) TraceLoggingHexInt32(_tlg_ENSURE_CONST(EventTag, eventTag), "_ms_EventTag")

/*
Macro TraceLoggingDescription(description):
Wrapper macro for setting a description for an event.

LTTNG semantics: TraceLoggingDescription has no effect.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingDescription("My event's detailed description"),
        TraceLoggingWideString(myString));
*/
#define TraceLoggingDescription(description) _tlg_ArgIgnored()

/*
Macro TraceLoggingCustomAttribute(key, value):
Wrapper macro for adding custom information about an event to the symbols.

LTTNG semantics: TraceLoggingCustomAttribute has no effect.

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingCustomAttribute("Key", "Value"),
        TraceLoggingWideString(myString));

Both parameters must be string literals. Multiple custom attributes can be
specified per event.
*/
#define TraceLoggingCustomAttribute(key, value) _tlg_ArgIgnored()

/*
Macro TraceLoggingStruct(fieldCount, "structName", "description", tags):
Wrapper macro for defining a group of related fields in an event.

LTTNG semantics: TraceLoggingStruct is ignored (struct fields become top-level
fields).

The description and tags parameters are optional.

The fieldCount parameter must be a compile-time constant. It indicates the
number of fields that will be considered to be part of the struct. A struct
and all of its contained fields count as a single field in any parent
structs.

The name parameter must be a string literal (not a variable) and must not
contain any '\0' characters.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Example:

    TraceLoggingWrite(MyProvider, "MyEventName",
        TraceLoggingStruct(2, "Name"),
        TraceLoggingWideString(szLast),
        TraceLoggingWideString(szFirst));
*/
#define TraceLoggingStruct(fieldCount, name, ...) _tlg_ArgIgnored()

#if defined(__cplusplus)
/*
Macro TraceLoggingValue(value, "name", "description", tags):
Wrapper macro for event fields. Automatically deduces value type. C++ only.

The name, description, and tags parameters are optional.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Examples:
- TraceLoggingValue(val1)                      // field name = "val1", description = unset,  tags = 0.
- TraceLoggingValue(val1, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingValue(val1, "name", "desc"       // field name = "name", description = "desc", tags = 0.
- TraceLoggingValue(val1, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.

Based on the type of val, TraceLoggingValue(val, ...) is equivalent to one of
the following, with the N of TraceLoggingIntN chosen based on sizeof(val):
- bool               --> TraceLoggingBoolean(val, ...)
- char               --> TraceLoggingChar(val, ...)
- char16_t           --> TraceLoggingChar16(val, ...)
- char32_t           --> TraceLoggingChar32(val, ...)
- wchar_t            --> TraceLoggingWChar(val, ...)
- signed   char      --> TraceLoggingIntN(val, ...)
- unsigned char      --> TraceLoggingUIntN(val, ...)
- signed   short     --> TraceLoggingIntN(val, ...)
- unsigned short     --> TraceLoggingUIntN(val, ...)
- signed   int       --> TraceLoggingIntN(val, ...)
- unsigned int       --> TraceLoggingUIntN(val, ...)
- signed   long      --> TraceLoggingIntN(val, ...)
- unsigned long      --> TraceLoggingUIntN(val, ...)
- signed   long long --> TraceLoggingIntN(val, ...)
- unsigned long long --> TraceLoggingUIntN(val, ...)
- float              --> TraceLoggingFloat32(val, ...)
- double             --> TraceLoggingFloat64(val, ...)
- const void*        --> TraceLoggingPointer(val, ...)    // Logs the pointer's value, not the data at which it points.
- const char*        --> TraceLoggingString(val, ...)     // Assumes nul-terminated utf-8 string.
- const char16_t*    --> TraceLoggingString16(val, ...)   // Assumes nul-terminated utf-16 string.
- const char32_t*    --> TraceLoggingString32(val, ...)   // Assumes nul-terminated utf-32 string.
- const wchar_t*     --> TraceLoggingWideString(val, ...) // Assumes nul-terminated utf-16 or utf-32 string, based on sizeof(wchar_t).
*/
#define TraceLoggingValue(value, ...) (_tlg_Value, value, _tlg_NDT(TraceLoggingValue, value, __VA_ARGS__))
#endif // __cplusplus

/*
Wrapper macros for event fields with simple scalar values.
Usage: TraceLoggingInt32(value, "name", "description", tags).

The name, description, and tags parameters are optional.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Notes:
- TraceLoggingBool is for 32-bit boolean values (e.g. int).
- TraceLoggingBoolean is for 8-bit boolean values (e.g. bool).

LTTNG-specific:
- TraceLoggingChar expects an ASCII character. Use
  TraceLoggingChar16((uint8_t)value) if your character might be 128-255.
- TraceLoggingChar16, TraceLoggingChar32, and TraceLoggingWChar will convert
  the character to a UTF-8 string for logging.

Examples:
- TraceLoggingInt32(val1)                      // field name = "val1", description = unset,  tags = 0.
- TraceLoggingInt32(val1, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingInt32(val1, "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingInt32(val1, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingInt8(value, ...) _tlg_ArgInt(int8_t, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt8, value, __VA_ARGS__))
#define TraceLoggingUInt8(value, ...) _tlg_ArgInt(uint8_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt8, value, __VA_ARGS__))
#define TraceLoggingInt16(value, ...) _tlg_ArgInt(int16_t, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt16, value, __VA_ARGS__))
#define TraceLoggingUInt16(value, ...) _tlg_ArgInt(uint16_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt16, value, __VA_ARGS__))
#define TraceLoggingInt32(value, ...) _tlg_ArgInt(int32_t, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt32, value, __VA_ARGS__))
#define TraceLoggingUInt32(value, ...) _tlg_ArgInt(uint32_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt32, value, __VA_ARGS__))
#define TraceLoggingLong(value, ...) _tlg_ArgInt(long, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingLong, value, __VA_ARGS__))
#define TraceLoggingULong(value, ...) _tlg_ArgInt(unsigned long, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingULong, value, __VA_ARGS__))
#define TraceLoggingInt64(value, ...) _tlg_ArgInt(int64_t, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt64, value, __VA_ARGS__))
#define TraceLoggingUInt64(value, ...) _tlg_ArgInt(uint64_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt64, value, __VA_ARGS__))
#define TraceLoggingHexInt8(value, ...) _tlg_ArgInt(int8_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt8, value, __VA_ARGS__))
#define TraceLoggingHexUInt8(value, ...) _tlg_ArgInt(uint8_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt8, value, __VA_ARGS__))
#define TraceLoggingHexInt16(value, ...) _tlg_ArgInt(int16_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt16, value, __VA_ARGS__))
#define TraceLoggingHexUInt16(value, ...) _tlg_ArgInt(uint16_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt16, value, __VA_ARGS__))
#define TraceLoggingHexInt32(value, ...) _tlg_ArgInt(int32_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt32, value, __VA_ARGS__))
#define TraceLoggingHexUInt32(value, ...) _tlg_ArgInt(uint32_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt32, value, __VA_ARGS__))
#define TraceLoggingHexLong(value, ...) _tlg_ArgInt(long, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexLong, value, __VA_ARGS__))
#define TraceLoggingHexULong(value, ...) _tlg_ArgInt(unsigned long, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexULong, value, __VA_ARGS__))
#define TraceLoggingHexInt64(value, ...) _tlg_ArgInt(int64_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt64, value, __VA_ARGS__))
#define TraceLoggingHexUInt64(value, ...) _tlg_ArgInt(uint64_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt64, value, __VA_ARGS__))
#define TraceLoggingIntPtr(value, ...) _tlg_ArgInt(intptr_t, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingIntPtr, value, __VA_ARGS__))
#define TraceLoggingUIntPtr(value, ...) _tlg_ArgInt(uintptr_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUIntPtr, value, __VA_ARGS__))
#define TraceLoggingFloat32(value, ...) _tlg_ArgFloat(float, value, _tlg_NDT(TraceLoggingFloat32, value, __VA_ARGS__))
#define TraceLoggingFloat64(value, ...) _tlg_ArgFloat(double, value, _tlg_NDT(TraceLoggingFloat64, value, __VA_ARGS__))
#define TraceLoggingBool(value, ...) _tlg_ArgEnum(int32_t, value, _tlgDecimal, 1, _tlg_NDT(TraceLoggingBool, value, __VA_ARGS__), lttngh_BoolEnumDesc)
#define TraceLoggingBoolean(value, ...) _tlg_ArgEnum(uint8_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingBoolean, value, __VA_ARGS__), lttngh_BoolEnumDesc)
#define TraceLoggingChar(value, ...) _tlg_ArgChar8(char, value, _tlg_NDT(TraceLoggingChar, value, __VA_ARGS__))
#define TraceLoggingChar16(value, ...) _tlg_ArgCharNN(char16_t, value, 16, _tlg_NDT(TraceLoggingChar16, value, __VA_ARGS__))
#define TraceLoggingChar32(value, ...) _tlg_ArgCharNN(char32_t, value, 32, _tlg_NDT(TraceLoggingChar32, value, __VA_ARGS__))
#define TraceLoggingWChar(value, ...) _tlg_ArgCharNN(wchar_t, value, _tlgWcharNN, _tlg_NDT(TraceLoggingWChar, value, __VA_ARGS__))
#define TraceLoggingPointer(value, ...) _tlg_ArgInt(void const *, value, _tlgHex, 0, _tlg_NDT(TraceLoggingPointer, value, __VA_ARGS__))
#define TraceLoggingCodePointer(value, ...) _tlg_ArgInt(void const *, value, _tlgHex, 0, _tlg_NDT(TraceLoggingCodePointer, value, __VA_ARGS__))
#define TraceLoggingPid(value, ...) _tlg_ArgInt(uint32_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingPid, value, __VA_ARGS__))
#define TraceLoggingTid(value, ...) _tlg_ArgInt(uint32_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingTid, value, __VA_ARGS__))
#define TraceLoggingPort(value, ...) _tlg_ArgInt(uint16_t, value, _tlgDecimalBE, 0, _tlg_NDT(TraceLoggingPort, value, __VA_ARGS__))
#define TraceLoggingWinError(value, ...) _tlg_ArgInt(uint32_t, value, _tlgDecimal, 0, _tlg_NDT(TraceLoggingWinError, value, __VA_ARGS__))
#define TraceLoggingNTStatus(value, ...) _tlg_ArgInt(int32_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingNTStatus, value, __VA_ARGS__))
#define TraceLoggingHResult(value, ...) _tlg_ArgInt(int32_t, value, _tlgHex, 0, _tlg_NDT(TraceLoggingHResult, value, __VA_ARGS__))

/*
Wrapper macros for event fields with complex scalar values.
Usage: TraceLoggingGuid(value, "name", "description", tags).

The name, description, and tags parameters are optional.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

LTTNG-specific:
- Because these values do not have well-defined types on Linux, the macros
  will accept nearly anything for the value parameter (no type checking). The
  macros just assume the value has the appropriate data. Basically, the
  macro expects that &(value) will resolve to a pointer to the correct number
  of bytes of data.
- In C, value must be an lvalue (i.e. &(value) must be a valid expression).
  In C++, value can be an rvalue.
- GUID will be logged as an array of 16 hexadecimal bytes.
- SYSTEMTIME will be logged as an array of 8 decimal ushorts (16 bytes total).
- FILETIME will be logged as an array of 1 decimal uint64.

Examples:
- TraceLoggingGuid(val1)                      // field name = "val1", description = unset,  tags = 0.
- TraceLoggingGuid(val1, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingGuid(val1, "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingGuid(val1, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingGuid(value, ...) _tlg_ArgIntArrayByRef(uint8_t, value, 16, _tlgHex, 0, _tlg_NDT(TraceLoggingGuid, value, __VA_ARGS__))
#define TraceLoggingSystemTime(value, ...) _tlg_ArgIntArrayByRef(uint16_t, value, 8, _tlgDecimal, 0, _tlg_NDT(TraceLoggingSystemTime, value, __VA_ARGS__))
#define TraceLoggingSystemTimeUtc(value, ...) _tlg_ArgIntArrayByRef(uint16_t, value, 8, _tlgDecimal, 0, _tlg_NDT(TraceLoggingSystemTimeUtc, value, __VA_ARGS__))
#define TraceLoggingFileTime(value, ...) _tlg_ArgIntArrayByRef(uint64_t, value, 1, _tlgDecimal, 0, _tlg_NDT(TraceLoggingFileTime, value, __VA_ARGS__))
#define TraceLoggingFileTimeUtc(value, ...) _tlg_ArgIntArrayByRef(uint64_t, value, 1, _tlgDecimal, 0, _tlg_NDT(TraceLoggingFileTimeUtc, value, __VA_ARGS__))

/*
Wrapper macros for event fields with string values.
Usage: TraceLoggingString(pszVal, "name", "description", tags), where pszVal is const char*.
Usage: TraceLoggingString16(pszVal, "name", "description", tags), where pszVal is const char16_t*.
Usage: TraceLoggingString32(pszVal, "name", "description", tags), where pszVal is const char32_t*.
Usage: TraceLoggingWideString(pszVal, "name", "description", tags), where pszVal is const wchar_t*.
Usage: TraceLoggingCountedString(pchVal, cchVal, "name", "description", tags), where pchVal is const char*.
Usage: TraceLoggingCountedString16(pchVal, cchVal, "name", "description", tags), where pchVal is const char16_t*.
Usage: TraceLoggingCountedString32(pchVal, cchVal, "name", "description", tags), where pchVal is const char32_t*.
Usage: TraceLoggingCountedWideString(pchVal, cchVal, "name", "description", tags), where pchVal is const wchar_t*.

The name, description, and tags parameters are optional.

For TraceLoggingString, TraceLoggingString16, TraceLoggingString32, and
TraceLoggingWideString, the pszValue parameter is treated as a nul-terminated
string. If pszValue is NULL, it is treated as an empty (zero-length) string.

For TraceLoggingCountedString, TraceLoggingCountedString16,
TraceLoggingCountedString32, and TraceLoggingCountedWideString, the pchValue
parameter is treated as a counted string, with length cchValue given in
characters. The pchValue parameter may be NULL only if cchValue is 0.

The name, description, and tags parameters are optional.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

LTTNG-specific:
- TraceLoggingString and TraceLoggingCountedString expect UTF-8 data.
- The remaining macros expect UTF-16 or UTF-32 data, which will be transcoded
  to UTF-8 before being written to the trace.

Examples:
- TraceLoggingString(psz1)                      // field name = "psz1", description = unset,  tags = 0.
- TraceLoggingString(psz1, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingString(psz1, "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingString(psz1, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingString(pszValue, ...) _tlg_ArgString8(char, pszValue, _tlg_NDT(TraceLoggingString, pszValue, __VA_ARGS__))
#define TraceLoggingString16(pszValue, ...) _tlg_ArgStringNN(char16_t, pszValue, 16, _tlg_NDT(TraceLoggingString16, pszValue, __VA_ARGS__))
#define TraceLoggingString32(pszValue, ...) _tlg_ArgStringNN(char32_t, pszValue, 32, _tlg_NDT(TraceLoggingString32, pszValue, __VA_ARGS__))
#define TraceLoggingWideString(pszValue, ...) _tlg_ArgStringNN(wchar_t, pszValue, _tlgWcharNN, _tlg_NDT(TraceLoggingWideString, pszValue, __VA_ARGS__))
#define TraceLoggingCountedString(pchValue, cchValue, ...) _tlg_ArgCountedString8(char, pchValue, cchValue, _tlg_NDT(TraceLoggingCountedString, pchValue, __VA_ARGS__))
#define TraceLoggingCountedString16(pchValue, cchValue, ...) _tlg_ArgCountedStringNN(char16_t, pchValue, cchValue, 16, _tlg_NDT(TraceLoggingCountedString16, pchValue, __VA_ARGS__))
#define TraceLoggingCountedString32(pchValue, cchValue, ...) _tlg_ArgCountedStringNN(char32_t, pchValue, cchValue, 32, _tlg_NDT(TraceLoggingCountedString32, pchValue, __VA_ARGS__))
#define TraceLoggingCountedWideString(pchValue, cchValue, ...) _tlg_ArgCountedStringNN(wchar_t, pchValue, cchValue, _tlgWcharNN, _tlg_NDT(TraceLoggingCountedWideString, pchValue, __VA_ARGS__))

/*
Wrapper macro for raw binary data.
Usage: TraceLoggingBinary(pValue, cbValue, "name", "description", tags).

LTTNG semantics: logged as an array of hexadecimal bytes.

The pValue parameter is treated as a const void* so that any kind of data can
be provided. The cbValue parameter is the size of the data, in bytes.

The name, description, and tags parameters are optional.

The pValue parameter may be NULL only if cbValue is 0.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Examples:
- TraceLoggingBinary(pObj, sizeof(*pObj))                      // field name = "pObj", description = unset,  tags = 0.
- TraceLoggingBinary(pObj, sizeof(*pObj), "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingBinary(pObj, sizeof(*pObj), "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingBinary(pObj, sizeof(*pObj), "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingBinary(pValue, cbValue, ...) _tlg_ArgBinary(void, pValue, cbValue, _tlg_NDT(TraceLoggingBinary, pValue, __VA_ARGS__))

/*
Wrapper macros for event fields with PSOCKADDR, PSOCKADDR_IN, etc. values.
Usage: TraceLoggingSocketAddress(pSockAddr, cbSockAddr, "name", "description", tags).

LTTNG semantics: logged as an array of hexadecimal bytes.

Note that the amount of data needed for a SOCKADDR field varies depending on
the type of address. If the data is stored in a union variable, be sure to
set the cbSockAddr parameter to the size of the correct union member or the
data might be truncated.

The name, description, and tags parameters are optional.

The pValue parameter may be NULL only if cbValue is 0. No type-checking is
performed -- any pointer is accepted.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

  Examples:
- TraceLoggingSocketAddress(pSock, sizeof(*pSock))                      // field name = "pSock", description = unset,  tags = 0.
- TraceLoggingSocketAddress(pSock, sizeof(*pSock), "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingSocketAddress(pSock, sizeof(*pSock), "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingSocketAddress(pSock, sizeof(*pSock), "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingSocketAddress(pValue, cbValue, ...) _tlg_ArgBinary(void, pValue, cbValue, _tlg_NDT(TraceLoggingSocketAddress, pValue, __VA_ARGS__))

/*
Wrapper macros for event fields with PSID values.
Usage: TraceLoggingSid(pSid, "name", "description", tags).

LTTNG semantics: logged as an array of hexadecimal bytes.

Note that the amount of data needed for a SID field varies depending on
the number of subauthorities. TraceLogging assumes the SID structure is valid
and will send the amount of data indicated by the subauthority count.

The name, description, and tags parameters are optional.

The pSid parameter must not be NULL and must point at a valid SID
(SubAuthorityCount must be initialized).

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

LTTNG-specific:
- Because SID values does not have a well-defined type on Linux, the macro
  will accept nearly anything for the pValue parameter (no type checking).
  Basically, the macro expects that pValue will resolve to a pointer to the
  correct number of bytes of data.

Examples:
- TraceLoggingSid(pSid)                      // field name = "pSid", description = unset,  tags = 0.
- TraceLoggingSid(pSid, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingSid(pSid, "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingSid(pSid, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingSid(pValue, ...) _tlg_ArgSid(void, pValue, _tlg_NDT(TraceLoggingSid, pValue, __VA_ARGS__))

/*
Wrapper macro for binary data referenced by a structure (advanced scenarios).
Usage: TraceLoggingBinaryBuffer(pBuffer, MyStructType, "name", "description", tags).

LTTNG semantics: logged as an array of hexadecimal bytes.

This macro supports serialization of structures that have fields "Buffer" and
"Length", where the Buffer field points to the data to be transmitted and the
Length field contains the number of bytes to be transmitted.

The name, description, and tags parameters are optional.

The pBuffer parameter must not be NULL.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Requirements:

- The pBuffer parameter must be a non-null pointer to BufferType (or const
  BufferType).
- The BufferType type must have fields "Length" and "Buffer".
  - The Length field must contain the size of the data (in bytes).
  - The Buffer field must contain a pointer to the data. (Buffer may be null if
    Length is 0.)
*/
#define TraceLoggingBinaryBuffer(pBuffer, BufferType, ...) _tlg_ArgBuffer(BufferType, pBuffer, _tlg_NDT(TraceLoggingBinaryBuffer, pBuffer, __VA_ARGS__))

/*
Wrapper macros for event fields with values that are fixed-length arrays.
Usage: TraceLoggingInt32FixedArray(pVals, cVals, "name", "description", tags).

The cVals parameter must be a compile-time constant value. It must not be 0.

The name, description, and tags parameters are optional.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Examples:
- TraceLoggingUInt8FixedArray(pbX1, 32)                      // field name = "pbX1", description = unset,  tags = 0.
- TraceLoggingUInt8FixedArray(pbX1, 32, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingUInt8FixedArray(pbX1, 32, "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingUInt8FixedArray(pbX1, 32, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingInt8FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int8_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt8FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingUInt8FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint8_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt8FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingInt16FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int16_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt16FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingUInt16FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint16_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt16FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingInt32FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int32_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt32FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingUInt32FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint32_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt32FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingLongFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(long, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingLongFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingULongFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(unsigned long, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingULongFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingInt64FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int64_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt64FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingUInt64FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint64_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt64FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexInt8FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int8_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt8FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt8FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint8_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt8FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexInt16FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int16_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt16FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt16FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint16_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt16FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexInt32FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int32_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt32FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt32FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint32_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt32FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexLongFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(long, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexLongFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexULongFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(unsigned long, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexULongFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexInt64FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int64_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt64FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt64FixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint64_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt64FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingIntPtrFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(intptr_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingIntPtrFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingUIntPtrFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uintptr_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUIntPtrFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingBoolFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(int32_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingBoolFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingBooleanFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint8_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingBooleanFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingCharFixedArray(pValues, cValues, ...) _tlg_ArgCountedString8(char, pValues, cValues, _tlg_NDT(TraceLoggingCharFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingChar16FixedArray(pValues, cValues, ...) _tlg_ArgCountedStringNN(char16_t, pValues, cValues, 16, _tlg_NDT(TraceLoggingChar16FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingChar32FixedArray(pValues, cValues, ...) _tlg_ArgCountedStringNN(char32_t, pValues, cValues, 32, _tlg_NDT(TraceLoggingChar32FixedArray, pValues, __VA_ARGS__))
#define TraceLoggingWCharFixedArray(pValues, cValues, ...) _tlg_ArgCountedStringNN(wchar_t, pValues, cValues, _tlgWcharNN, _tlg_NDT(TraceLoggingWCharFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingPointerFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(void const *, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingPointerFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingCodePointerFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(void const *, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingCodePointerFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingGuidFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint8_t, (uint8_t const *)(pValues), (cValues)*16u, _tlgDecimal, 0, _tlg_NDT(TraceLoggingGuidFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingFileTimeFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint64_t, (uint64_t const *)(pValues), cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingFileTimeFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingFileTimeUtcFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint64_t, (uint64_t const *)(pValues), cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingFileTimeUtcFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingSystemTimeFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint16_t, (uint16_t const *)(pValues), (cValues)*8u, _tlgDecimal, 0, _tlg_NDT(TraceLoggingSystemTimeFixedArray, pValues, __VA_ARGS__))
#define TraceLoggingSystemTimeUtcFixedArray(pValues, cValues, ...) _tlg_ArgIntFixedArray(uint16_t, (uint16_t const *)(pValues), (cValues)*8u, _tlgDecimal, 0, _tlg_NDT(TraceLoggingSystemTimeUtcFixedArray, pValues, __VA_ARGS__))

// Broken (LTTNG-UST does not handle metadata for array-of-float):
//#define TraceLoggingFloat32FixedArray(pValues, cValues, ...)          _tlg_ArgFloatFixedArray(float,  pValues, cValues,                 _tlg_NDT(TraceLoggingFloat32FixedArray, pValues, __VA_ARGS__))
//#define TraceLoggingFloat64FixedArray(pValues, cValues, ...)          _tlg_ArgFloatFixedArray(double, pValues, cValues,                 _tlg_NDT(TraceLoggingFloat64FixedArray, pValues, __VA_ARGS__))

/*
Wrapper macros for event fields with values that are variable-length arrays.
Usage: TraceLoggingInt32Array(pVals, cVals, "name", "description", tags).

The name, description, and tags parameters are optional.

The pointer parameter may be NULL only if the count parameter is 0.

If provided, the name parameter must be a string literal (not a variable) and
must not contain any '\0' characters. If the name is not provided, the value
parameter is used to automatically generate a name.

LTTNG-specific:
- Field name must contain only letters, numbers, and '_'.
- Field name must be unique within the event.
- Violations of these rules cannot be detected by TraceLoggingProvider.h, but
  will likely cause problems when you try to decode the trace.
- Be especially careful with automatically-generated field names, as they
  often contain spaces other problematic symbols.

If provided, the description parameter must be a string literal.
(Field description is ignored for LTTNG.)

If provided, the tags parameter must be an integer value.
(Field tags are ignored for LTTNG.)

Examples:
- TraceLoggingUInt8Array(pbX1, cbX1)                      // field name = "pbX1", description = unset,  tags = 0.
- TraceLoggingUInt8Array(pbX1, cbX1, "name")              // field name = "name", description = unset,  tags = 0.
- TraceLoggingUInt8Array(pbX1, cbX1, "name", "desc")      // field name = "name", description = "desc", tags = 0.
- TraceLoggingUInt8Array(pbX1, cbX1, "name", "desc", 0x4) // field name = "name", description = "desc", tags = 0x4.
*/
#define TraceLoggingInt8Array(pValues, cValues, ...) _tlg_ArgIntArray(int8_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt8Array, pValues, __VA_ARGS__))
#define TraceLoggingUInt8Array(pValues, cValues, ...) _tlg_ArgIntArray(uint8_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt8Array, pValues, __VA_ARGS__))
#define TraceLoggingInt16Array(pValues, cValues, ...) _tlg_ArgIntArray(int16_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt16Array, pValues, __VA_ARGS__))
#define TraceLoggingUInt16Array(pValues, cValues, ...) _tlg_ArgIntArray(uint16_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt16Array, pValues, __VA_ARGS__))
#define TraceLoggingInt32Array(pValues, cValues, ...) _tlg_ArgIntArray(int32_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt32Array, pValues, __VA_ARGS__))
#define TraceLoggingUInt32Array(pValues, cValues, ...) _tlg_ArgIntArray(uint32_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt32Array, pValues, __VA_ARGS__))
#define TraceLoggingLongArray(pValues, cValues, ...) _tlg_ArgIntArray(long, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingLongArray, pValues, __VA_ARGS__))
#define TraceLoggingULongArray(pValues, cValues, ...) _tlg_ArgIntArray(unsigned long, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingULongArray, pValues, __VA_ARGS__))
#define TraceLoggingInt64Array(pValues, cValues, ...) _tlg_ArgIntArray(int64_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingInt64Array, pValues, __VA_ARGS__))
#define TraceLoggingUInt64Array(pValues, cValues, ...) _tlg_ArgIntArray(uint64_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUInt64Array, pValues, __VA_ARGS__))
#define TraceLoggingHexInt8Array(pValues, cValues, ...) _tlg_ArgIntArray(int8_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt8Array, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt8Array(pValues, cValues, ...) _tlg_ArgIntArray(uint8_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt8Array, pValues, __VA_ARGS__))
#define TraceLoggingHexInt16Array(pValues, cValues, ...) _tlg_ArgIntArray(int16_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt16Array, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt16Array(pValues, cValues, ...) _tlg_ArgIntArray(uint16_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt16Array, pValues, __VA_ARGS__))
#define TraceLoggingHexInt32Array(pValues, cValues, ...) _tlg_ArgIntArray(int32_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt32Array, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt32Array(pValues, cValues, ...) _tlg_ArgIntArray(uint32_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt32Array, pValues, __VA_ARGS__))
#define TraceLoggingHexLongArray(pValues, cValues, ...) _tlg_ArgIntArray(long, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexLongArray, pValues, __VA_ARGS__))
#define TraceLoggingHexULongArray(pValues, cValues, ...) _tlg_ArgIntArray(unsigned long, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexULongArray, pValues, __VA_ARGS__))
#define TraceLoggingHexInt64Array(pValues, cValues, ...) _tlg_ArgIntArray(int64_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexInt64Array, pValues, __VA_ARGS__))
#define TraceLoggingHexUInt64Array(pValues, cValues, ...) _tlg_ArgIntArray(uint64_t, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingHexUInt64Array, pValues, __VA_ARGS__))
#define TraceLoggingIntPtrArray(pValues, cValues, ...) _tlg_ArgIntArray(intptr_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingIntPtrArray, pValues, __VA_ARGS__))
#define TraceLoggingUIntPtrArray(pValues, cValues, ...) _tlg_ArgIntArray(uintptr_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingUIntPtrArray, pValues, __VA_ARGS__))
#define TraceLoggingBoolArray(pValues, cValues, ...) _tlg_ArgIntArray(int32_t, pValues, cValues, _tlgDecimal, 1, _tlg_NDT(TraceLoggingBoolArray, pValues, __VA_ARGS__))
#define TraceLoggingBooleanArray(pValues, cValues, ...) _tlg_ArgIntArray(uint8_t, pValues, cValues, _tlgDecimal, 0, _tlg_NDT(TraceLoggingBooleanArray, pValues, __VA_ARGS__))
#define TraceLoggingCharArray(pValues, cValues, ...) _tlg_ArgCountedString8(char, pValues, cValues, _tlg_NDT(TraceLoggingCharArray, pValues, __VA_ARGS__))
#define TraceLoggingChar16Array(pValues, cValues, ...) _tlg_ArgCountedStringNN(char16_t, pValues, cValues, 16, _tlg_NDT(TraceLoggingChar16Array, pValues, __VA_ARGS__))
#define TraceLoggingChar32Array(pValues, cValues, ...) _tlg_ArgCountedStringNN(char32_t, pValues, cValues, 32, _tlg_NDT(TraceLoggingChar32Array, pValues, __VA_ARGS__))
#define TraceLoggingWCharArray(pValues, cValues, ...) _tlg_ArgCountedStringNN(wchar_t, pValues, cValues, _tlgWcharNN, _tlg_NDT(TraceLoggingWCharArray, pValues, __VA_ARGS__))
#define TraceLoggingPointerArray(pValues, cValues, ...) _tlg_ArgIntArray(void const *, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingPointerArray, pValues, __VA_ARGS__))
#define TraceLoggingCodePointerArray(pValues, cValues, ...) _tlg_ArgIntArray(void const *, pValues, cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingCodePointerArray, pValues, __VA_ARGS__))
#define TraceLoggingGuidArray(pValues, cValues, ...) _tlg_ArgIntArray(uint8_t, (uint8_t const *)(pValues), (cValues)*16u, _tlgDecimal, 0, _tlg_NDT(TraceLoggingGuidArray, pValues, __VA_ARGS__))
#define TraceLoggingFileTimeArray(pValues, cValues, ...) _tlg_ArgIntArray(uint64_t, (uint64_t const *)(pValues), cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingFileTimeArray, pValues, __VA_ARGS__))
#define TraceLoggingFileTimeUtcArray(pValues, cValues, ...) _tlg_ArgIntArray(uint64_t, (uint64_t const *)(pValues), cValues, _tlgHex, 0, _tlg_NDT(TraceLoggingFileTimeUtcArray, pValues, __VA_ARGS__))
#define TraceLoggingSystemTimeArray(pValues, cValues, ...) _tlg_ArgIntArray(uint16_t, (uint16_t const *)(pValues), (cValues)*8u, _tlgDecimal, 0, _tlg_NDT(TraceLoggingSystemTimeArray, pValues, __VA_ARGS__))
#define TraceLoggingSystemTimeUtcArray(pValues, cValues, ...) _tlg_ArgIntArray(uint16_t, (uint16_t const *)(pValues), (cValues)*8u, _tlgDecimal, 0, _tlg_NDT(TraceLoggingSystemTimeUtcArray, pValues, __VA_ARGS__))

    // Broken (LTTNG-UST does not handle metadata for sequence-of-float):
    //#define TraceLoggingFloat32Array(pValues, cValues, ...)          _tlg_ArgFloatArray(float,  pValues, cValues,                 _tlg_NDT(TraceLoggingFloat32Array, pValues, __VA_ARGS__))
    //#define TraceLoggingFloat64Array(pValues, cValues, ...)          _tlg_ArgFloatArray(double, pValues, cValues,                 _tlg_NDT(TraceLoggingFloat64Array, pValues, __VA_ARGS__))

#ifdef __EDG__
#pragma endregion
#endif

#ifdef __EDG__
#pragma region Internal utility macros
#endif

#ifndef _tlg_ASSERT
#define _tlg_ASSERT(x) assert(x)
#endif // _tlg_ASSERT

#define _tlg_FLATTEN(...) __VA_ARGS__
#define _tlg_PARENTHESIZE(...) (__VA_ARGS__)

#define _tlg_STRINGIZE_imp(x) #x
#define _tlg_STRINGIZE(x) _tlg_STRINGIZE_imp(x)

#define _tlg_PASTE_imp(a, b) a##b
#define _tlg_PASTE(a, b) _tlg_PASTE_imp(a, b)

#define _tlg_CAT_imp(a, ...) a##__VA_ARGS__
#define _tlg_CAT(a, ...) _tlg_CAT_imp(a, __VA_ARGS__)

#define _tlg_SPLIT_imp0(false_val, ...) false_val
#define _tlg_SPLIT_imp1(false_val, ...) __VA_ARGS__
#define _tlg_SPLIT_imp(cond, args) _tlg_PASTE(_tlg_SPLIT_imp, cond) args
#define _tlg_SPLIT(cond, ...) _tlg_SPLIT_imp(cond, (__VA_ARGS__))

#define _tlg_IS_PARENTHESIZED_imp0(...) 1
#define _tlg_IS_PARENTHESIZED_imp1 1,
#define _tlg_IS_PARENTHESIZED_imp_tlg_IS_PARENTHESIZED_imp0 0,
#define _tlg_IS_PARENTHESIZED(...) \
    _tlg_SPLIT(0, _tlg_CAT(_tlg_IS_PARENTHESIZED_imp, _tlg_IS_PARENTHESIZED_imp0 __VA_ARGS__))

#define _tlg_IS_EMPTY(...) _tlg_SPLIT(                      \
    _tlg_IS_PARENTHESIZED(__VA_ARGS__),                     \
    _tlg_IS_PARENTHESIZED(_tlg_PARENTHESIZE __VA_ARGS__()), \
    0)

#define _tlg_NARGS_imp2(                              \
    a1, a2, a3, a4, a5, a6, a7, a8, a9,               \
    a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, \
    a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, \
    a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, \
    a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, \
    a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, \
    a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, \
    a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, \
    a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, \
    a90, a91, a92, a93, a94, a95, a96, a97, a98, a99, \
    size, ...) size
#define _tlg_NARGS_imp0(...) _tlg_PASTE(_tlg_NARGS_imp2(__VA_ARGS__,                            \
                                                        99, 98, 97, 96, 95, 94, 93, 92, 91, 90, \
                                                        89, 88, 87, 86, 85, 84, 83, 82, 81, 80, \
                                                        79, 78, 77, 76, 75, 74, 73, 72, 71, 70, \
                                                        69, 68, 67, 66, 65, 64, 63, 62, 61, 60, \
                                                        59, 58, 57, 56, 55, 54, 53, 52, 51, 50, \
                                                        49, 48, 47, 46, 45, 44, 43, 42, 41, 40, \
                                                        39, 38, 37, 36, 35, 34, 33, 32, 31, 30, \
                                                        29, 28, 27, 26, 25, 24, 23, 22, 21, 20, \
                                                        19, 18, 17, 16, 15, 14, 13, 12, 11, 10, \
                                                        9, 8, 7, 6, 5, 4, 3, 2, 1, ), )
#define _tlg_NARGS_imp1() 0
#define _tlg_NARGS_imp(is_empty, args) _tlg_PASTE(_tlg_NARGS_imp, is_empty) args
#define _tlg_NARGS(...) _tlg_NARGS_imp(_tlg_IS_EMPTY(__VA_ARGS__), (__VA_ARGS__))

#define _tlg_FOR_imp0(f, ...)
#define _tlg_FOR_imp1(f, a0) f(0, a0)
#define _tlg_FOR_imp2(f, a0, a1) f(0, a0) f(1, a1)
#define _tlg_FOR_imp3(f, a0, a1, a2) f(0, a0) f(1, a1) f(2, a2)
#define _tlg_FOR_imp4(f, a0, a1, a2, a3) f(0, a0) f(1, a1) f(2, a2) f(3, a3)
#define _tlg_FOR_imp5(f, a0, a1, a2, a3, a4) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4)
#define _tlg_FOR_imp6(f, a0, a1, a2, a3, a4, a5) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5)
#define _tlg_FOR_imp7(f, a0, a1, a2, a3, a4, a5, a6) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6)
#define _tlg_FOR_imp8(f, a0, a1, a2, a3, a4, a5, a6, a7) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7)
#define _tlg_FOR_imp9(f, a0, a1, a2, a3, a4, a5, a6, a7, a8) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8)
#define _tlg_FOR_imp10(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9)
#define _tlg_FOR_imp11(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10)
#define _tlg_FOR_imp12(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11)
#define _tlg_FOR_imp13(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12)
#define _tlg_FOR_imp14(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13)
#define _tlg_FOR_imp15(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14)
#define _tlg_FOR_imp16(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15)
#define _tlg_FOR_imp17(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16)
#define _tlg_FOR_imp18(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17)
#define _tlg_FOR_imp19(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18)
#define _tlg_FOR_imp20(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19)
#define _tlg_FOR_imp21(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20)
#define _tlg_FOR_imp22(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21)
#define _tlg_FOR_imp23(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22)
#define _tlg_FOR_imp24(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23)
#define _tlg_FOR_imp25(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24)
#define _tlg_FOR_imp26(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25)
#define _tlg_FOR_imp27(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26)
#define _tlg_FOR_imp28(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27)
#define _tlg_FOR_imp29(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28)
#define _tlg_FOR_imp30(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29)
#define _tlg_FOR_imp31(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30)
#define _tlg_FOR_imp32(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31)
#define _tlg_FOR_imp33(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32)
#define _tlg_FOR_imp34(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33)
#define _tlg_FOR_imp35(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34)
#define _tlg_FOR_imp36(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35)
#define _tlg_FOR_imp37(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36)
#define _tlg_FOR_imp38(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37)
#define _tlg_FOR_imp39(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38)
#define _tlg_FOR_imp40(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39)
#define _tlg_FOR_imp41(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40)
#define _tlg_FOR_imp42(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41)
#define _tlg_FOR_imp43(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42)
#define _tlg_FOR_imp44(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43)
#define _tlg_FOR_imp45(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44)
#define _tlg_FOR_imp46(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45)
#define _tlg_FOR_imp47(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46)
#define _tlg_FOR_imp48(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47)
#define _tlg_FOR_imp49(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48)
#define _tlg_FOR_imp50(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49)
#define _tlg_FOR_imp51(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50)
#define _tlg_FOR_imp52(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51)
#define _tlg_FOR_imp53(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52)
#define _tlg_FOR_imp54(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53)
#define _tlg_FOR_imp55(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54)
#define _tlg_FOR_imp56(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55)
#define _tlg_FOR_imp57(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56)
#define _tlg_FOR_imp58(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57)
#define _tlg_FOR_imp59(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58)
#define _tlg_FOR_imp60(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59)
#define _tlg_FOR_imp61(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60)
#define _tlg_FOR_imp62(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61)
#define _tlg_FOR_imp63(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62)
#define _tlg_FOR_imp64(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63)
#define _tlg_FOR_imp65(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64)
#define _tlg_FOR_imp66(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65)
#define _tlg_FOR_imp67(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66)
#define _tlg_FOR_imp68(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67)
#define _tlg_FOR_imp69(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68)
#define _tlg_FOR_imp70(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69)
#define _tlg_FOR_imp71(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70)
#define _tlg_FOR_imp72(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71)
#define _tlg_FOR_imp73(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72)
#define _tlg_FOR_imp74(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73)
#define _tlg_FOR_imp75(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74)
#define _tlg_FOR_imp76(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75)
#define _tlg_FOR_imp77(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76)
#define _tlg_FOR_imp78(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77)
#define _tlg_FOR_imp79(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78)
#define _tlg_FOR_imp80(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79)
#define _tlg_FOR_imp81(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80)
#define _tlg_FOR_imp82(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81)
#define _tlg_FOR_imp83(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82)
#define _tlg_FOR_imp84(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83)
#define _tlg_FOR_imp85(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84)
#define _tlg_FOR_imp86(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85)
#define _tlg_FOR_imp87(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86)
#define _tlg_FOR_imp88(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87)
#define _tlg_FOR_imp89(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88)
#define _tlg_FOR_imp90(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89)
#define _tlg_FOR_imp91(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90)
#define _tlg_FOR_imp92(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91)
#define _tlg_FOR_imp93(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92)
#define _tlg_FOR_imp94(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92, a93) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92) f(93, a93)
#define _tlg_FOR_imp95(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92, a93, a94) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92) f(93, a93) f(94, a94)
#define _tlg_FOR_imp96(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92, a93, a94, a95) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92) f(93, a93) f(94, a94) f(95, a95)
#define _tlg_FOR_imp97(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92, a93, a94, a95, a96) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92) f(93, a93) f(94, a94) f(95, a95) f(96, a96)
#define _tlg_FOR_imp98(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92, a93, a94, a95, a96, a97) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92) f(93, a93) f(94, a94) f(95, a95) f(96, a96) f(97, a97)
#define _tlg_FOR_imp99(f, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22, a23, a24, a25, a26, a27, a28, a29, a30, a31, a32, a33, a34, a35, a36, a37, a38, a39, a40, a41, a42, a43, a44, a45, a46, a47, a48, a49, a50, a51, a52, a53, a54, a55, a56, a57, a58, a59, a60, a61, a62, a63, a64, a65, a66, a67, a68, a69, a70, a71, a72, a73, a74, a75, a76, a77, a78, a79, a80, a81, a82, a83, a84, a85, a86, a87, a88, a89, a90, a91, a92, a93, a94, a95, a96, a97, a98) f(0, a0) f(1, a1) f(2, a2) f(3, a3) f(4, a4) f(5, a5) f(6, a6) f(7, a7) f(8, a8) f(9, a9) f(10, a10) f(11, a11) f(12, a12) f(13, a13) f(14, a14) f(15, a15) f(16, a16) f(17, a17) f(18, a18) f(19, a19) f(20, a20) f(21, a21) f(22, a22) f(23, a23) f(24, a24) f(25, a25) f(26, a26) f(27, a27) f(28, a28) f(29, a29) f(30, a30) f(31, a31) f(32, a32) f(33, a33) f(34, a34) f(35, a35) f(36, a36) f(37, a37) f(38, a38) f(39, a39) f(40, a40) f(41, a41) f(42, a42) f(43, a43) f(44, a44) f(45, a45) f(46, a46) f(47, a47) f(48, a48) f(49, a49) f(50, a50) f(51, a51) f(52, a52) f(53, a53) f(54, a54) f(55, a55) f(56, a56) f(57, a57) f(58, a58) f(59, a59) f(60, a60) f(61, a61) f(62, a62) f(63, a63) f(64, a64) f(65, a65) f(66, a66) f(67, a67) f(68, a68) f(69, a69) f(70, a70) f(71, a71) f(72, a72) f(73, a73) f(74, a74) f(75, a75) f(76, a76) f(77, a77) f(78, a78) f(79, a79) f(80, a80) f(81, a81) f(82, a82) f(83, a83) f(84, a84) f(85, a85) f(86, a86) f(87, a87) f(88, a88) f(89, a89) f(90, a90) f(91, a91) f(92, a92) f(93, a93) f(94, a94) f(95, a95) f(96, a96) f(97, a97) f(98, a98)

#define _tlg_FOR_imp(n, macroAndArgs) _tlg_PASTE(_tlg_FOR_imp, n) macroAndArgs
#define _tlg_FOREACH(macro, ...) _tlg_FOR_imp(_tlg_NARGS(__VA_ARGS__), (macro, __VA_ARGS__))

#ifdef __EDG__
#pragma endregion
#endif

#ifdef __EDG__
#pragma region Internal declarations
#endif

    // For TraceLogging-internal use only.
    struct _tlg_Provider_t
    {
        struct lttng_probe_desc ProbeDesc;
        int IsRegistered;
    };

    // For TraceLogging-internal use only.
    struct _tlg_Event_t
    {
        struct lttng_event_desc Desc; // MUST be first field.
        uint64_t Keyword;
        struct lttng_ust_tracepoint const *ProbePtr;
        char const *Name;
        int const *LevelPtr;
        int Level;
    };

#ifdef __EDG__
#pragma endregion
#endif

#ifdef __EDG__
#pragma region Internal helper functions
#endif

    unsigned _tlg_EventFullName(
        char *pFullName,
        char const *pchProviderName,
        unsigned cchProviderName,
        char const *pchEventName,
        unsigned cchEventName,
        uint64_t keyword)
        _tlg_NOEXCEPT _tlg_WEAK_ATTRIBUTES;
    unsigned _tlg_EventFullName(
        char *pFullName,
        char const *pchProviderName,
        unsigned cchProviderName,
        char const *pchEventName,
        unsigned cchEventName,
        uint64_t keyword)
        _tlg_NOEXCEPT
    {
        char *pOut = pFullName;
        char const *const pOutEnd = pOut + (LTTNG_UST_SYM_NAME_LEN - 1);

        // ProviderName + ":" + EventName must be less than 256 characters.
        if (cchProviderName + 1 + cchEventName >= LTTNG_UST_SYM_NAME_LEN)
        {
            // If not, truncate event name.
            _tlg_ASSERT(!"ProviderName+EventName too long");
            cchEventName = (LTTNG_UST_SYM_NAME_LEN - 2) - cchProviderName;
        }

        memcpy(pOut, pchProviderName, cchProviderName);
        pOut += cchProviderName;
        *pOut++ = ':';
        pOut = (char *)memcpy(pOut, pchEventName, cchEventName);
        pOut += cchEventName;

        if (keyword != 0)
        {
            // ProviderName + ":" + EventName + KeywordSuffix must be less than 256 characters.
            if (pOut == pOutEnd)
            {
                // If not, truncate KeywordSuffix.
                _tlg_ASSERT(!"ProviderName+EventName too long");
            }
            else
            {
                unsigned k = 0;
                *pOut++ = ';';
                do
                {
                    if (keyword & 1)
                    {
                        // ProviderName + ":" + EventName + KeywordSuffix must be less than 256 characters.
                        if (pOutEnd - pOut < 4)
                        {
                            // If not, truncate KeywordSuffix.
                            _tlg_ASSERT(!"ProviderName+EventName too long");
                            break;
                        }

                        *pOut++ = 'k';
                        if (k < 10)
                        {
                            *pOut++ = (char)(k + '0');
                        }
                        else
                        {
                            *pOut++ = (char)(k / 10 + '0');
                            *pOut++ = (char)(k % 10 + '0');
                        }
                        *pOut++ = ';';
                    }

                    keyword >>= 1;
                    k += 1;
                } while (keyword != 0);
            }
        }

        *pOut = 0;
        return (unsigned)(pOut - pFullName);
    }

    int _tlg_EventEnabled(
        struct _tlg_Provider_t *pProvider,
        char const *eventName,
        struct lttng_event_desc const **pEventDescStart,
        struct lttng_event_desc const **pEventDescStop,
        int const **ppState)
        _tlg_NOEXCEPT _tlg_WEAK_ATTRIBUTES;
    int _tlg_EventEnabled(
        struct _tlg_Provider_t *pProvider,
        char const *eventName,
        struct lttng_event_desc const **pEventDescStart,
        struct lttng_event_desc const **pEventDescStop,
        int const **ppState)
        _tlg_NOEXCEPT
    {
        static const int NullState = 0;
        int state = 0;

        if (__atomic_load_n(&pProvider->IsRegistered, __ATOMIC_ACQUIRE) == 1)
        {
            char pchEventFullName[LTTNG_UST_SYM_NAME_LEN];
            unsigned const cchEventFullName = _tlg_EventFullName(
                pchEventFullName,
                pProvider->ProbeDesc.provider, (unsigned)strlen(pProvider->ProbeDesc.provider),
                eventName, (unsigned)strlen(eventName),
                0); // Don't add keyword suffix.

            for (struct lttng_event_desc const **ppDesc = pEventDescStart; ppDesc != pEventDescStop; ppDesc += 1)
            {
                struct _tlg_Event_t *pEvent = (struct _tlg_Event_t *)*ppDesc;
                if (pEvent != NULL &&
                    0 == strncmp(pchEventFullName, pEvent->Desc.name, cchEventFullName) && // Ignore keyword suffix.
                    (pEvent->Desc.name[cchEventFullName] == 0 || pEvent->Desc.name[cchEventFullName] == ';'))
                {
                    __atomic_store_n(ppState, &pEvent->ProbePtr->state, __ATOMIC_RELAXED);
                    state = CMM_LOAD_SHARED(pEvent->ProbePtr->state);
                    goto Done;
                }
            }

            _tlg_ASSERT(!"TraceLoggingEventEnabled called with invalid event name");
            __atomic_store_n(ppState, &NullState, __ATOMIC_RELAXED);
        }

    Done:

        return state;
    }

    int _tlg_ProviderUnregister(
        struct _tlg_Provider_t *pProvider,
        struct lttng_ust_tracepoint *const *pTracepointStart)
        _tlg_NOEXCEPT _tlg_WEAK_ATTRIBUTES;
    int _tlg_ProviderUnregister(
        struct _tlg_Provider_t *pProvider,
        struct lttng_ust_tracepoint *const *pTracepointStart)
        _tlg_NOEXCEPT
    {
        return lttngh_UnregisterProvider(
            &pProvider->IsRegistered,
            &pProvider->ProbeDesc,
            pTracepointStart);
    }

    int _tlg_ProviderRegister(
        struct _tlg_Provider_t *pProvider,
        struct lttng_ust_tracepoint **pTracepointStart,
        struct lttng_ust_tracepoint **pTracepointStop,
        struct lttng_event_desc const **pEventDescStart,
        struct lttng_event_desc const **pEventDescStop)
        _tlg_NOEXCEPT _tlg_WEAK_ATTRIBUTES;
    int _tlg_ProviderRegister(
        struct _tlg_Provider_t *pProvider,
        struct lttng_ust_tracepoint **pTracepointStart,
        struct lttng_ust_tracepoint **pTracepointStop,
        struct lttng_event_desc const **pEventDescStart,
        struct lttng_event_desc const **pEventDescStop)
        _tlg_NOEXCEPT
    {
        // Fix up event names, then call lttngh_RegisterProvider.

        char const *const pchProvName = pProvider->ProbeDesc.provider;
        unsigned const cchProvName = (unsigned)strlen(pchProvName);

        if (__atomic_exchange_n(&pProvider->IsRegistered, 2, __ATOMIC_RELAXED) != 0)
        {
            // Called TraceLoggingRegister on an already-registered handle.
            // (Or memory corruption?)
            abort();
        }

        for (struct lttng_event_desc const **ppDesc = pEventDescStart;
             ppDesc != pEventDescStop; ppDesc += 1)
        {
            struct _tlg_Event_t *pEvent = (struct _tlg_Event_t *)*ppDesc;
            if (pEvent != NULL && pEvent->Desc.name[0] == 0)
            {
                _tlg_EventFullName(
                    (char *)pEvent->Desc.name,
                    pchProvName, cchProvName,
                    pEvent->Name, (unsigned)strlen(pEvent->Name),
                    pEvent->Keyword);
            }
        }

        if (__atomic_exchange_n(&pProvider->IsRegistered, 0, __ATOMIC_RELEASE) != 2)
        {
            // Should never happen. (Memory corruption?)
            abort();
        }

        return lttngh_RegisterProvider(
            &pProvider->IsRegistered,
            &pProvider->ProbeDesc,
            pTracepointStart,
            pTracepointStop,
            pEventDescStart,
            pEventDescStop);
    }

    static inline char const *_tlg_ProviderName(struct _tlg_Provider_t *pProvider) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
    static inline char const *_tlg_ProviderName(struct _tlg_Provider_t *pProvider) _tlg_NOEXCEPT
    {
        return pProvider->ProbeDesc.provider;
    }

    static inline uint16_t _tlg_SidSize(void const *pSid) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
    static inline uint16_t _tlg_SidSize(void const *pSid) _tlg_NOEXCEPT
    {
        uint8_t const *p = (uint8_t const *)pSid;
        return (uint16_t)(8u + p[1] * 4u);
    }

    static inline void _tlg_DataDescCreateArray(
        struct lttngh_DataDesc *pDesc,
        void const *pVals,
        uint16_t cVals,
        unsigned cbVal,
        unsigned char alignment)
        _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
    static inline void _tlg_DataDescCreateArray(
        struct lttngh_DataDesc *pDesc,
        void const *pVals,
        uint16_t cVals,
        unsigned cbVal,
        unsigned char alignment)
        _tlg_NOEXCEPT
    {
        uint16_t const *const pLength = &pDesc[1].Length;
        pDesc[0] = lttngh_DataDescCreate(pLength, sizeof(uint16_t), lttng_alignof(uint16_t), lttngh_DataType_None);
        pDesc[1] = lttngh_DataDescCreateCounted(pVals, cVals * cbVal, alignment, cVals);
    }

    static inline void _tlg_DataDescCreateTinyArray(
        struct lttngh_DataDesc *pDesc,
        void const *pVals,
        uint8_t cVals,
        unsigned cbVal,
        unsigned char alignment)
        _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
    static inline void _tlg_DataDescCreateTinyArray(
        struct lttngh_DataDesc *pDesc,
        void const *pVals,
        uint8_t cVals,
        unsigned cbVal,
        unsigned char alignment)
        _tlg_NOEXCEPT
    {
        uint8_t const *const pLength = (uint8_t const *)&pDesc[1].Length + (BYTE_ORDER == BIG_ENDIAN);
        pDesc[0] = lttngh_DataDescCreate(pLength, sizeof(uint8_t), lttng_alignof(uint8_t), lttngh_DataType_None);
        pDesc[1] = lttngh_DataDescCreateCounted(pVals, cVals * cbVal, alignment, cVals);
    }

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#ifdef __EDG__
#pragma endregion
#endif

#ifdef __EDG__
#pragma region Internal implementation macros
#endif

#define _tlg_IntDataSigned0 lttngh_DataType_Unsigned
#define _tlg_IntDataSigned1 lttngh_DataType_Signed

#define _tlg_IntFieldType_tlgDecimal \
    0, 10, lttng_encode_none, {} // byteswap, radix, encoding, padding
#define _tlg_IntFieldType_tlgHex \
    0, 16, lttng_encode_none, {} // byteswap, radix, encoding, padding
#if BYTE_ORDER == LITTLE_ENDIAN
#define _tlg_IntFieldType_tlgDecimalBE \
    1, 10, lttng_encode_none, {} // byteswap, radix, encoding, padding
#else
#define _tlg_IntFieldType_tlgDecimalBE \
    0, 10, lttng_encode_none, {} // byteswap, radix, encoding, padding
#endif
#define _tlg_IntFieldType_tlgUTF8 \
    0, 10, lttng_encode_UTF8, {} // byteswap, radix, encoding, padding

#define _tlg_FloatFieldType_float \
    8, 24, 8u * lttng_alignof(float), 0, {} // exponent, mantissa, align, byteswap, padding
#define _tlg_FloatFieldType_double \
    11, 53, 8u * lttng_alignof(double), 0, {} // exponent, mantissa, align, byteswap, padding

#ifdef __cplusplus

// Remove reference
template <class T>
struct _tlgRemoveReference
{
    typedef T type;
};
template <class T>
struct _tlgRemoveReference<T &>
{
    typedef T type;
};
template <class T>
struct _tlgRemoveReference<T &&>
{
    typedef T type;
};

// Remove const/volatile
template <class T>
struct _tlgRemoveCV
{
    typedef T type;
};
template <class T>
struct _tlgRemoveCV<T const>
{
    typedef T type;
};
template <class T>
struct _tlgRemoveCV<T volatile>
{
    typedef T type;
};
template <class T>
struct _tlgRemoveCV<T const volatile>
{
    typedef T type;
};

// Given non-ref type, remove const/volatile.
template <class T>
struct _tlgDecay_impl
{
    typedef typename _tlgRemoveCV<T>::type type;
};
template <class T>
struct _tlgDecay_impl<T[]>
{
    typedef T *type;
};
template <class T, size_t n>
struct _tlgDecay_impl<T[n]>
{
    typedef T *type;
};

// Remove reference, remove const/volatile, arrays decay to pointers.
template <class T>
struct _tlgDecay
{
    typedef typename _tlgDecay_impl<typename _tlgRemoveReference<T>::type>::type type;
};

// Data descriptor creation (general):

template <class ctype>
inline void _tlgCppInit1Desc(struct lttngh_DataDesc &desc, ctype const &value, lttngh_DataType type) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1Desc(struct lttngh_DataDesc &desc, ctype const &value, lttngh_DataType type) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreate(&value, sizeof(ctype), lttng_alignof(ctype), type);
}

template <class ctype, uint16_t cValues, class T>
inline void _tlgCppInit1DescByRef(struct lttngh_DataDesc &desc, T const &value) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype, uint16_t cValues, class T>
inline void _tlgCppInit1DescByRef(struct lttngh_DataDesc &desc, T const &value) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateCounted(&value, cValues * sizeof(ctype), lttng_alignof(ctype), cValues);
}

template <class ctype, uint16_t cValues>
inline void _tlgCppInit1DescFixedArray(struct lttngh_DataDesc &desc, ctype const *pValues) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype, uint16_t cValues>
inline void _tlgCppInit1DescFixedArray(struct lttngh_DataDesc &desc, ctype const *pValues) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateCounted(pValues, cValues * sizeof(ctype), lttng_alignof(ctype), cValues);
}

template <class ctype>
inline void _tlgCppInit1DescString8(struct lttngh_DataDesc &desc, ctype const *pszValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescString8(struct lttngh_DataDesc &desc, ctype const *pszValue) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char), "Wrong char size");
    desc = lttngh_DataDescCreateString8(pszValue ? (char const *)pszValue : "");
}

template <class ctype>
inline void _tlgCppInit1DescString16(struct lttngh_DataDesc &desc, ctype const *pszValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescString16(struct lttngh_DataDesc &desc, ctype const *pszValue) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char16_t), "Wrong char size");
    desc = lttngh_DataDescCreateStringUtf16(pszValue ? (char16_t const *)pszValue : u"");
}

template <class ctype>
inline void _tlgCppInit1DescString32(struct lttngh_DataDesc &desc, ctype const *pszValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescString32(struct lttngh_DataDesc &desc, ctype const *pszValue) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char32_t), "Wrong char size");
    desc = lttngh_DataDescCreateStringUtf32(pszValue ? (char32_t const *)pszValue : U"");
}

template <class ctype>
inline void _tlgCppInit1DescSeqUtf16(struct lttngh_DataDesc &desc, ctype const *pchValue, uint16_t cchValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescSeqUtf16(struct lttngh_DataDesc &desc, ctype const *pchValue, uint16_t cchValue) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char16_t), "Wrong char size");
    desc = lttngh_DataDescCreateSequenceUtf16((char16_t const *)pchValue, cchValue);
}

template <class ctype>
inline void _tlgCppInit1DescSeqUtf32(struct lttngh_DataDesc &desc, ctype const *pchValue, uint16_t cchValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescSeqUtf32(struct lttngh_DataDesc &desc, ctype const *pchValue, uint16_t cchValue) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char32_t), "Wrong char size");
    desc = lttngh_DataDescCreateSequenceUtf32((char32_t const *)pchValue, cchValue);
}

template <class ctype>
inline void _tlgCppInit1DescCharUtf16(struct lttngh_DataDesc &desc, ctype const &value) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescCharUtf16(struct lttngh_DataDesc &desc, ctype const &value) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char16_t), "Wrong char size");
    desc = lttngh_DataDescCreateSequenceUtf16((char16_t const *)&value, 1);
}

template <class ctype>
inline void _tlgCppInit1DescCharUtf32(struct lttngh_DataDesc &desc, ctype const &value) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit1DescCharUtf32(struct lttngh_DataDesc &desc, ctype const &value) _tlg_NOEXCEPT
{
    static_assert(sizeof(ctype) == sizeof(char32_t), "Wrong char size");
    desc = lttngh_DataDescCreateSequenceUtf32((char32_t const *)&value, 1);
}

template <class ctype, unsigned cbValue = sizeof(ctype), unsigned char alignment = lttng_alignof(ctype)>
inline void _tlgCppInit2DescArray(struct lttngh_DataDesc *pDesc, ctype const *pValues, uint16_t cValues) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype, unsigned cbValue, unsigned char alignment>
inline void _tlgCppInit2DescArray(struct lttngh_DataDesc *pDesc, ctype const *pValues, uint16_t cValues) _tlg_NOEXCEPT
{
    _tlg_DataDescCreateArray(pDesc, pValues, cValues, cbValue, alignment);
}

template <class ctype, unsigned cbValue = sizeof(uint8_t), unsigned char alignment = lttng_alignof(uint8_t)>
inline void _tlgCppInit2DescBuffer(struct lttngh_DataDesc *pDesc, ctype const *pValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype, unsigned cbValue, unsigned char alignment>
inline void _tlgCppInit2DescBuffer(struct lttngh_DataDesc *pDesc, ctype const *pValue) _tlg_NOEXCEPT
{
    _tlg_DataDescCreateArray(pDesc, pValue->Buffer, pValue->Length / cbValue, cbValue, alignment);
}

template <class ctype>
inline void _tlgCppInit2DescSid(struct lttngh_DataDesc *pDesc, ctype const *pValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit2DescSid(struct lttngh_DataDesc *pDesc, ctype const *pValue) _tlg_NOEXCEPT
{
    _tlg_DataDescCreateArray(pDesc, pValue, _tlg_SidSize(pValue), sizeof(uint8_t), lttng_alignof(uint8_t));
}

template <class ctype>
inline void _tlgCppInit2DescGuidPtr(struct lttngh_DataDesc *pDesc, ctype const *pValue) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class ctype>
inline void _tlgCppInit2DescGuidPtr(struct lttngh_DataDesc *pDesc, ctype const *pValue) _tlg_NOEXCEPT
{
    _tlg_DataDescCreateTinyArray(pDesc, pValue, pValue ? 16 : 0, sizeof(uint8_t), lttng_alignof(uint8_t));
}

// Data descriptor creation (TraceLoggingValue):

template <unsigned size, unsigned align, bool isSigned>
struct _tlgTypeMapInt
{
    static constexpr lttng_type _tlgLttngType =
        {atype_integer, {.basic = {.integer = {8u * size, 8u * align, isSigned, _tlg_IntFieldType_tlgDecimal}}}};

    static const lttngh_DataType _tlgType = isSigned ? lttngh_DataType_Signed : lttngh_DataType_Unsigned;
    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

template <unsigned size, unsigned align>
struct _tlgTypeMapHexInt
{
    static constexpr lttng_type _tlgLttngType =
        {atype_integer, {.basic = {.integer = {8u * size, 8u * align, 0, _tlg_IntFieldType_tlgHex}}}};

    static const lttngh_DataType _tlgType = lttngh_DataType_Unsigned;
    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

struct _tlgTypeMapUtf8String
{
    static constexpr lttng_type _tlgLttngType =
        {atype_string, {.basic = {.string = {lttng_encode_UTF8}}}};

    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

struct _tlgTypeMapUtf8Sequence
{
    static constexpr lttng_type _tlgLttngType =
        {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(char), 8u * lttng_alignof(char), 0, _tlg_IntFieldType_tlgUTF8}}}}}}};

    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

template <class T>
struct _tlgTypeMapBase
{
    static_assert(sizeof(T) == 0, "The type is not supported by TraceLoggingValue.");
};

template <class T>
struct _tlgTypeMap : _tlgTypeMapBase<typename _tlgDecay<T>::type>
{
};

// The following type maps are covered by the template _tlgCppInit1DescAuto:

#define _tlgTypeMapDeclInt(T)                                                               \
    template <>                                                                             \
    struct _tlgTypeMapBase<signed T> : _tlgTypeMapInt<sizeof(T), lttng_alignof(T), true>    \
    {                                                                                       \
    };                                                                                      \
    template <>                                                                             \
    struct _tlgTypeMapBase<unsigned T> : _tlgTypeMapInt<sizeof(T), lttng_alignof(T), false> \
    {                                                                                       \
    }
_tlgTypeMapDeclInt(char);
_tlgTypeMapDeclInt(short);
_tlgTypeMapDeclInt(int);
_tlgTypeMapDeclInt(long);
_tlgTypeMapDeclInt(long long);
#undef _tlgTypeMapDeclInt

template <>
struct _tlgTypeMapBase<void *> : _tlgTypeMapHexInt<sizeof(void *), lttng_alignof(void *)>
{
};
template <>
struct _tlgTypeMapBase<void const *> : _tlgTypeMapHexInt<sizeof(void const *), lttng_alignof(void const *)>
{
};

template <>
struct _tlgTypeMapBase<bool>
{
    static constexpr lttng_type _tlgLttngType =
        {atype_enum, {.basic = {.enumeration =
#ifdef USE_LTTNG_2_7
                                {.name = "bool" }
#else
                                    {&lttngh_BoolEnumDesc, {8u * sizeof(bool), 8u * lttng_alignof(bool), 0, _tlg_IntFieldType_tlgDecimal}}
#endif
                      }}};

    static const lttngh_DataType _tlgType = lttngh_DataType_Unsigned;
    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

template <>
struct _tlgTypeMapBase<float>
{
    static constexpr lttng_type _tlgLttngType =
        {atype_float, {.basic = {._float = {_tlg_FloatFieldType_float}}}};

    static const lttngh_DataType _tlgType = lttngh_DataType_Float;
    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

template <>
struct _tlgTypeMapBase<double>
{
    static constexpr lttng_type _tlgLttngType =
        {atype_float, {.basic = {._float = {_tlg_FloatFieldType_double}}}};

    static const lttngh_DataType _tlgType = lttngh_DataType_Float;
    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

template <class T>
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, T const &val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
template <class T>
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, T const &val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreate(&val, sizeof(T), lttng_alignof(T), _tlgTypeMap<T>::_tlgType);
}

// The following type maps are covered by special-case overrides of _tlgCppInit1DescAuto.

template <>
struct _tlgTypeMapBase<char const *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char const *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char const *val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateString8(val ? val : "");
}

template <>
struct _tlgTypeMapBase<char *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char *val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateString8(val ? val : "");
}

template <>
struct _tlgTypeMapBase<char16_t const *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char16_t const *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char16_t const *val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateStringUtf16(val ? val : u"");
}

template <>
struct _tlgTypeMapBase<char16_t *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char16_t *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char16_t *val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateStringUtf16(val ? val : u"");
}

template <>
struct _tlgTypeMapBase<char32_t const *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char32_t const *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char32_t const *val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateStringUtf32(val ? val : U"");
}

template <>
struct _tlgTypeMapBase<char32_t *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char32_t *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char32_t *val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateStringUtf32(val ? val : U"");
}

template <>
struct _tlgTypeMapBase<wchar_t const *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, wchar_t const *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, wchar_t const *val) _tlg_NOEXCEPT
{
    static_assert(sizeof(_tlgWcharNN_type) == sizeof(wchar_t), "Incorrectly-detected wchar_t size.");
    desc = _tlg_PASTE(lttngh_DataDescCreateStringUtf, _tlgWcharNN)(
        reinterpret_cast<_tlgWcharNN_type const *>(val ? val : L""));
}

template <>
struct _tlgTypeMapBase<wchar_t *> : _tlgTypeMapUtf8String
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, wchar_t *val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, wchar_t *val) _tlg_NOEXCEPT
{
    static_assert(sizeof(_tlgWcharNN_type) == sizeof(wchar_t), "Incorrectly-detected wchar_t size.");
    desc = _tlg_PASTE(lttngh_DataDescCreateStringUtf, _tlgWcharNN)(
        reinterpret_cast<_tlgWcharNN_type const *>(val ? val : L""));
}

template <>
struct _tlgTypeMapBase<char>
{
    static constexpr lttng_type _tlgLttngType =
        {atype_array, {.array = {{atype_integer, {.basic = {.integer = {8u * sizeof(char), 8u * lttng_alignof(char), 0, _tlg_IntFieldType_tlgUTF8}}}}, 1}}};

    static const lttngh_DataType _tlgType = lttngh_DataType_Unsigned;
    static inline constexpr lttng_event_field _tlgField(char const *szName) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES
    {
        return {szName, _tlgLttngType, 0, {}};
    }
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char const &val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char const &val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateCounted(&val, sizeof(char), lttng_alignof(char), 1);
}

template <>
struct _tlgTypeMapBase<char16_t> : _tlgTypeMapUtf8Sequence
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char16_t const &val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char16_t const &val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateSequenceUtf16(&val, 1);
}

template <>
struct _tlgTypeMapBase<char32_t> : _tlgTypeMapUtf8Sequence
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char32_t const &val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, char32_t const &val) _tlg_NOEXCEPT
{
    desc = lttngh_DataDescCreateSequenceUtf32(&val, 1);
}

template <>
struct _tlgTypeMapBase<wchar_t> : _tlgTypeMapUtf8Sequence
{
};

inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, wchar_t const &val) _tlg_NOEXCEPT _tlg_INLINE_ATTRIBUTES;
inline void _tlgCppInit1DescAuto(struct lttngh_DataDesc &desc, wchar_t const &val) _tlg_NOEXCEPT
{
    static_assert(sizeof(_tlgWcharNN_type) == sizeof(wchar_t), "Incorrectly-detected wchar_t size.");
    desc = _tlg_PASTE(lttngh_DataDescCreateSequenceUtf, _tlgWcharNN)(
        reinterpret_cast<_tlgWcharNN_type const *>(&val), 1);
}

#define _tlgBeginCppEval (
#define _tlgEndCppEval )
#define _tlg_ENSURE_CONST(name, n) _tlgIntegral##name<(n)>::value
template <class T, T n>
struct _tlgIntegralConstant
{
    enum
    {
        value = n
    };
};
template <uint8_t n>
struct _tlgIntegralChannel : _tlgIntegralConstant<uint8_t, n>
{
};
template <uint8_t n>
struct _tlgIntegralLevel : _tlgIntegralConstant<uint8_t, n>
{
};
template <uint8_t n>
struct _tlgIntegralOpcode : _tlgIntegralConstant<uint8_t, n>
{
};
template <uint64_t n>
struct _tlgIntegralKeyword : _tlgIntegralConstant<uint64_t, n>
{
};
template <uint32_t n>
struct _tlgIntegralEventTag : _tlgIntegralConstant<uint32_t, n>
{
};
#else
#define _tlgBeginCppEval
#define _tlgEndCppEval
#define _tlg_ENSURE_CONST(name, n) (n)
#endif

#define _tlgParseProviderId(...) \
    _tlgParseProviderId_impN(_tlg_NARGS(__VA_ARGS__), __VA_ARGS__)
#define _tlgParseProviderId_impN(n, providerId) \
    _tlg_PASTE(_tlgParseProviderId_imp, n)(providerId)
#define _tlgParseProviderId_imp0(...) /* parameter not provided - error case */ \
    static_assert(0, "TRACELOGGING_DEFINE_PROVIDER providerId must be specified as eleven integers, e.g. (1,2,3,4,5,6,7,8,9,10,11).");
#define _tlgParseProviderId_imp1(providerId) \
    _tracelogging_SyntaxError_ProviderIdMustBeEnclosedInParentheses providerId
#define _tracelogging_SyntaxError_ProviderIdMustBeEnclosedInParentheses(...)                                                                          \
    static_assert(_tlg_NARGS(__VA_ARGS__) == 11, "TRACELOGGING_DEFINE_PROVIDER providerId must be eleven integers, e.g. (1,2,3,4,5,6,7,8,9,10,11)."); \
    static_assert(1 _tlg_FOREACH(_tlgParseProviderId_CheckInt, __VA_ARGS__), "TRACELOGGING_DEFINE_PROVIDER providerId must be eleven integers, e.g. (1,2,3,4,5,6,7,8,9,10,11).");
#define _tlgParseProviderId_CheckInt(n, val) +(val)

#define _tlg_NDT_imp0(pname, value, ...) (#value, , , 0)
#define _tlg_NDT_imp1(pname, value, name) (name, , , 0)
#define _tlg_NDT_imp2(pname, value, name, desc) (name, L##desc, , 0)
#define _tlg_NDT_imp3(pname, value, name, desc, ftags) (name, L##desc, ftags, 1)
#define _tlg_NDT_imp4(pname, ...) (too_many_values_passed_to_##pname, , , 0)
#define _tlg_NDT_imp5(pname, ...) (too_many_values_passed_to_##pname, , , 0)

#define _tlg_NDT_impB(macro, args) macro args
#define _tlg_NDT_impA(n, args) _tlg_NDT_impB(_tlg_PASTE(_tlg_NDT_imp, n), args)
#define _tlg_NDT(pname, value, ...) _tlg_NDT_impA(_tlg_NARGS(__VA_ARGS__), (pname, value, __VA_ARGS__))

#define _tlg_NDT_Name_impA(name, desc, ftags, haveTags) name
#define _tlg_NDT_Name(ndt) _tlg_NDT_Name_impA ndt

#define _tlg_ApplyUnwrap(...) __VA_ARGS__

#define _tlg_ApplyArgs_impB(baseMacro, handler, ...) baseMacro##handler(__VA_ARGS__)
#define _tlg_ApplyArgs_impA(baseMacro, ...) _tlg_ApplyArgs_impB(baseMacro, __VA_ARGS__)
#define _tlg_ApplyArgs(baseMacro, args) _tlg_ApplyArgs_impA(baseMacro, _tlg_ApplyUnwrap args)

#define _tlg_ApplyArgsN_impB(baseMacro, n, handler, ...) baseMacro##handler(n, __VA_ARGS__)
#define _tlg_ApplyArgsN_impA(baseMacro, n, ...) _tlg_ApplyArgsN_impB(baseMacro, n, __VA_ARGS__)
#define _tlg_ApplyArgsN(baseMacro, n, args) _tlg_ApplyArgsN_impA(baseMacro, n, _tlg_ApplyUnwrap args)

// FOREACH handlers

#define _tlg_FieldCount(n, args) _tlg_ApplyArgs(_tlg_FieldCount, args)
#define _tlg_FieldCount_tlg_Ignored()
#define _tlg_FieldCount_tlg_Level(eventLevel)
#define _tlg_FieldCount_tlg_Keyword(eventKeyword)
#define _tlg_FieldCount_tlg_Int(ctype, value, intFieldType, isSigned, ndt) +1
#define _tlg_FieldCount_tlg_Enum(ctype, value, etype, intFieldType, isSigned, ndt) +1
#define _tlg_FieldCount_tlg_IntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt) +1
#define _tlg_FieldCount_tlg_IntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) +1
#define _tlg_FieldCount_tlg_IntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) +1
#define _tlg_FieldCount_tlg_Float(ctype, value, ndt) +1
#define _tlg_FieldCount_tlg_FloatArray(ctype, pValues, cValues, ndt) +1
#define _tlg_FieldCount_tlg_FloatFixedArray(ctype, pValues, cValues, ndt) +1
#define _tlg_FieldCount_tlg_Char8(ctype, value, ndt) +1
#define _tlg_FieldCount_tlg_CharNN(ctype, value, NN, ndt) +1
#define _tlg_FieldCount_tlg_String8(ctype, pszValue, ndt) +1
#define _tlg_FieldCount_tlg_StringNN(ctype, pszValue, NN, ndt) +1
#define _tlg_FieldCount_tlg_CountedString8(ctype, pchValue, cchValue, ndt) +1
#define _tlg_FieldCount_tlg_CountedStringNN(ctype, pchValue, cchValue, NN, ndt) +1
#define _tlg_FieldCount_tlg_Binary(ctype, pValue, cbValue, ndt) +1
#define _tlg_FieldCount_tlg_Sid(ctype, pValue, ndt) +1
#define _tlg_FieldCount_tlg_GuidPtr(ctype, pValue, name) +1
#define _tlg_FieldCount_tlg_Buffer(ctype, pValue, ndt) +1
#define _tlg_FieldCount_tlg_Value(value, ndt) +1

#define _tlg_FieldInfo(n, args) _tlg_ApplyArgs(_tlg_FieldInfo, args)
#define _tlg_FieldInfo_tlg_Ignored()
#define _tlg_FieldInfo_tlg_Level(eventLevel)
#define _tlg_FieldInfo_tlg_Keyword(eventKeyword)
#define _tlg_FieldInfo_tlg_Int(ctype, value, intFieldType, isSigned, ndt)                                                                 \
    {_tlg_NDT_Name(ndt),                                                                                                                  \
     {atype_integer, {.basic = {.integer = {8u * sizeof(ctype), 8u * lttng_alignof(ctype), isSigned, _tlg_IntFieldType##intFieldType}}}}, \
     0,                                                                                                                                   \
     {}},
#define _tlg_FieldInfo_tlg_Enum(ctype, value, etype, intFieldType, isSigned, ndt)                                                                    \
    {_tlg_NDT_Name(ndt),                                                                                                                             \
     {atype_enum, {.basic = {.enumeration = {&etype, {8u * sizeof(ctype), 8u * lttng_alignof(ctype), isSigned, _tlg_IntFieldType##intFieldType}}}}}, \
     0,                                                                                                                                              \
     {}},
#define _tlg_FieldInfo_tlg_IntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt)                                                                                     \
    {_tlg_NDT_Name(ndt),                                                                                                                                                         \
     {atype_array, {.array = {{atype_integer, {.basic = {.integer = {8u * sizeof(ctype), 8u * lttng_alignof(ctype), isSigned, _tlg_IntFieldType##intFieldType}}}}, (cValues)}}}, \
     0,                                                                                                                                                                          \
     {}},
#define _tlg_FieldInfo_tlg_IntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt)                                                                                                                                                                                                                    \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                                     \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(ctype), 8u * lttng_alignof(ctype), isSigned, _tlg_IntFieldType##intFieldType}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                                      \
     {}},
#define _tlg_FieldInfo_tlg_IntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt)                                                                                   \
    {_tlg_NDT_Name(ndt),                                                                                                                                                         \
     {atype_array, {.array = {{atype_integer, {.basic = {.integer = {8u * sizeof(ctype), 8u * lttng_alignof(ctype), isSigned, _tlg_IntFieldType##intFieldType}}}}, (cValues)}}}, \
     0,                                                                                                                                                                          \
     {}},
#define _tlg_FieldInfo_tlg_Float(ctype, value, ndt)                       \
    {_tlg_NDT_Name(ndt),                                                  \
     {atype_float, {.basic = {._float = {_tlg_FloatFieldType_##ctype}}}}, \
     0,                                                                   \
     {}},
#define _tlg_FieldInfo_tlg_FloatArray(ctype, pValues, cValues, ndt)                                                                                                                                                                          \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                     \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_float, {.basic = {._float = {_tlg_FloatFieldType_##ctype}}}}}}}, \
     0,                                                                                                                                                                                                                                      \
     {}},
#define _tlg_FieldInfo_tlg_FloatFixedArray(ctype, pValues, cValues, ndt)                                         \
    {_tlg_NDT_Name(ndt),                                                                                         \
     {atype_array, {.array = {{atype_float, {.basic = {._float = {_tlg_FloatFieldType_##ctype}}}}, (cValues)}}}, \
     0,                                                                                                          \
     {}},
#define _tlg_FieldInfo_tlg_Char8(ctype, value, ndt)                                                                                                         \
    {_tlg_NDT_Name(ndt),                                                                                                                                    \
     {atype_array, {.array = {{atype_integer, {.basic = {.integer = {8u * sizeof(ctype), 8u * lttng_alignof(ctype), 0, _tlg_IntFieldType_tlgUTF8}}}}, 1}}}, \
     0,                                                                                                                                                     \
     {}},
#define _tlg_FieldInfo_tlg_CharNN(ctype, value, NN, ndt)                                                                                                                                                                                                                                      \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                      \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(char), 8u * lttng_alignof(char), 0, _tlg_IntFieldType_tlgUTF8}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                       \
     {}},
#define _tlg_FieldInfo_tlg_String8(ctype, pszValue, ndt)         \
    {_tlg_NDT_Name(ndt),                                         \
     {atype_string, {.basic = {.string = {lttng_encode_UTF8}}}}, \
     0,                                                          \
     {}},
#define _tlg_FieldInfo_tlg_StringNN(ctype, pszValue, NN, ndt)    \
    {_tlg_NDT_Name(ndt),                                         \
     {atype_string, {.basic = {.string = {lttng_encode_UTF8}}}}, \
     0,                                                          \
     {}},
#define _tlg_FieldInfo_tlg_CountedString8(ctype, pchValue, cchValue, ndt)                                                                                                                                                                                                                       \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                        \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(ctype), 8u * lttng_alignof(ctype), 0, _tlg_IntFieldType_tlgUTF8}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                         \
     {}},
#define _tlg_FieldInfo_tlg_CountedStringNN(ctype, pchValue, cchValue, NN, ndt)                                                                                                                                                                                                                \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                      \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(char), 8u * lttng_alignof(char), 0, _tlg_IntFieldType_tlgUTF8}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                       \
     {}},
#define _tlg_FieldInfo_tlg_Binary(ctype, pValue, cbValue, ndt)                                                                                                                                                                                                                                     \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                           \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(uint8_t), 8u * lttng_alignof(uint8_t), 0, _tlg_IntFieldType_tlgHex}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                            \
     {}},
#define _tlg_FieldInfo_tlg_Sid(ctype, pValue, ndt)                                                                                                                                                                                                                                                 \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                           \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(uint8_t), 8u * lttng_alignof(uint8_t), 0, _tlg_IntFieldType_tlgHex}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                            \
     {}},
#define _tlg_FieldInfo_tlg_GuidPtr(ctype, pValue, name)                                                                                                                                                                                                                                          \
    {name,                                                                                                                                                                                                                                                                                       \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint8_t), 8u * lttng_alignof(uint8_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(uint8_t), 8u * lttng_alignof(uint8_t), 0, _tlg_IntFieldType_tlgHex}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                          \
     {}},
#define _tlg_FieldInfo_tlg_Buffer(ctype, pValue, ndt)                                                                                                                                                                                                                                              \
    {_tlg_NDT_Name(ndt),                                                                                                                                                                                                                                                                           \
     {atype_sequence, {.sequence = {{atype_integer, {.basic = {.integer = {8u * sizeof(uint16_t), 8u * lttng_alignof(uint16_t), 0, _tlg_IntFieldType_tlgDecimal}}}}, {atype_integer, {.basic = {.integer = {8u * sizeof(uint8_t), 8u * lttng_alignof(uint8_t), 0, _tlg_IntFieldType_tlgHex}}}}}}}, \
     0,                                                                                                                                                                                                                                                                                            \
     {}},
#define _tlg_FieldInfo_tlg_Value(value, ndt) \
    _tlgTypeMap<decltype(value)>::_tlgField(_tlg_NDT_Name(ndt)),

#define _tlg_LevelVal(n, args) _tlg_ApplyArgs(_tlg_LevelVal, args)
#define _tlg_LevelVal_tlg_Ignored()
#define _tlg_LevelVal_tlg_Level(eventLevel) &0 )|( _tlg_ENSURE_CONST(Level, eventLevel)
#define _tlg_LevelVal_tlg_Keyword(eventKeyword)
#define _tlg_LevelVal_tlg_Int(ctype, value, intFieldType, isSigned, ndt)
#define _tlg_LevelVal_tlg_Enum(ctype, value, etype, intFieldType, isSigned, ndt)
#define _tlg_LevelVal_tlg_IntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt)
#define _tlg_LevelVal_tlg_IntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt)
#define _tlg_LevelVal_tlg_IntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt)
#define _tlg_LevelVal_tlg_Float(ctype, value, ndt)
#define _tlg_LevelVal_tlg_FloatArray(ctype, pValues, cValues, ndt)
#define _tlg_LevelVal_tlg_FloatFixedArray(ctype, pValues, cValues, ndt)
#define _tlg_LevelVal_tlg_Char8(ctype, value, ndt)
#define _tlg_LevelVal_tlg_CharNN(ctype, value, NN, ndt)
#define _tlg_LevelVal_tlg_String8(ctype, pszValue, ndt)
#define _tlg_LevelVal_tlg_StringNN(ctype, pszValue, NN, ndt)
#define _tlg_LevelVal_tlg_CountedString8(ctype, pchValue, cchValue, ndt)
#define _tlg_LevelVal_tlg_CountedStringNN(ctype, pchValue, cchValue, NN, ndt)
#define _tlg_LevelVal_tlg_Binary(ctype, pValue, cbValue, ndt)
#define _tlg_LevelVal_tlg_Sid(ctype, pValue, ndt)
#define _tlg_LevelVal_tlg_GuidPtr(ctype, pValue, name)
#define _tlg_LevelVal_tlg_Buffer(ctype, pValue, ndt)
#define _tlg_LevelVal_tlg_Value(value, ndt)

#define _tlg_KeywordVal(n, args) _tlg_ApplyArgs(_tlg_KeywordVal, args)
#define _tlg_KeywordVal_tlg_Ignored()
#define _tlg_KeywordVal_tlg_Level(eventLevel)
#define _tlg_KeywordVal_tlg_Keyword(eventKeyword) | _tlg_ENSURE_CONST(Keyword, eventKeyword)
#define _tlg_KeywordVal_tlg_Int(ctype, value, intFieldType, isSigned, ndt)
#define _tlg_KeywordVal_tlg_Enum(ctype, value, etype, intFieldType, isSigned, ndt)
#define _tlg_KeywordVal_tlg_IntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt)
#define _tlg_KeywordVal_tlg_IntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt)
#define _tlg_KeywordVal_tlg_IntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt)
#define _tlg_KeywordVal_tlg_Float(ctype, value, ndt)
#define _tlg_KeywordVal_tlg_FloatArray(ctype, pValues, cValues, ndt)
#define _tlg_KeywordVal_tlg_FloatFixedArray(ctype, pValues, cValues, ndt)
#define _tlg_KeywordVal_tlg_Char8(ctype, value, ndt)
#define _tlg_KeywordVal_tlg_CharNN(ctype, value, NN, ndt)
#define _tlg_KeywordVal_tlg_String8(ctype, pszValue, ndt)
#define _tlg_KeywordVal_tlg_StringNN(ctype, pszValue, NN, ndt)
#define _tlg_KeywordVal_tlg_CountedString8(ctype, pchValue, cchValue, ndt)
#define _tlg_KeywordVal_tlg_CountedStringNN(ctype, pchValue, cchValue, NN, ndt)
#define _tlg_KeywordVal_tlg_Binary(ctype, pValue, cbValue, ndt)
#define _tlg_KeywordVal_tlg_Sid(ctype, pValue, ndt)
#define _tlg_KeywordVal_tlg_GuidPtr(ctype, pValue, name)
#define _tlg_KeywordVal_tlg_Buffer(ctype, pValue, ndt)
#define _tlg_KeywordVal_tlg_Value(value, ndt)

#define _tlg_DataDescCount(n, args) _tlg_ApplyArgs(_tlg_DataDescCount, args)
#define _tlg_DataDescCount_tlg_Ignored()
#define _tlg_DataDescCount_tlg_Level(eventLevel)
#define _tlg_DataDescCount_tlg_Keyword(eventKeyword)
#define _tlg_DataDescCount_tlg_Int(ctype, value, intFieldType, isSigned, ndt) +1
#define _tlg_DataDescCount_tlg_Enum(ctype, value, etype, intFieldType, isSigned, ndt) +1
#define _tlg_DataDescCount_tlg_IntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt) +1
#define _tlg_DataDescCount_tlg_IntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) +2 // sequence
#define _tlg_DataDescCount_tlg_IntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) +1
#define _tlg_DataDescCount_tlg_Float(ctype, value, ndt) +1
#define _tlg_DataDescCount_tlg_FloatArray(ctype, pValues, cValues, ndt) +2 // sequence
#define _tlg_DataDescCount_tlg_FloatFixedArray(ctype, pValues, cValues, ndt) +1
#define _tlg_DataDescCount_tlg_Char8(ctype, value, ndt) +1
#define _tlg_DataDescCount_tlg_CharNN(ctype, value, NN, ndt) +1 // sequence (transcoded)
#define _tlg_DataDescCount_tlg_String8(ctype, pszValue, ndt) +1
#define _tlg_DataDescCount_tlg_StringNN(ctype, pszValue, NN, ndt) +1
#define _tlg_DataDescCount_tlg_CountedString8(ctype, pchValue, cchValue, ndt) +2      // sequence
#define _tlg_DataDescCount_tlg_CountedStringNN(ctype, pchValue, cchValue, NN, ndt) +1 // sequence (transcoded)
#define _tlg_DataDescCount_tlg_Binary(ctype, pValue, cbValue, ndt) +2                 // sequence
#define _tlg_DataDescCount_tlg_Sid(ctype, pValue, ndt) +2                             // sequence
#define _tlg_DataDescCount_tlg_GuidPtr(ctype, pValue, name) +2                        // sequence
#define _tlg_DataDescCount_tlg_Buffer(ctype, pValue, ndt) +2                          // sequence
#define _tlg_DataDescCount_tlg_Value(value, ndt) +1

#ifdef __cplusplus
/*
The C++ versions of the _tlg_DataDescCreate macros are different from the C
versions.

The primary motivation for this is the destruction of temporaries. In C++,
evaluating the user-supplied expressions and calling EventProbe must be a
single statement (no semicolons between evaluating any user-supplied
expressions and calling EventProbe). Otherwise, temporaries created by the
evaluation of the user's expressions are destroyed before invoking EventProbe,
meaning that the logged data is garbage. Temporaries are destroyed at the
semicolon, and we need the EventProbe to occur before the temporaries are
destroyed.

The most common example of this is when a user writes a function F() that
returns a std::string, then tries to log the string using F().c_str().

It is possible to deal with this using C-compatible code, but doing that
requires splitting the DataDescCreate macro into two loops (one to declare the
temporary variables and one to fill in the DataDesc objects), which makes them
harder to understand and maintain (as well as increases the work the compiler
must do to process the macros). In addition, using C++-specific techniques
in the DataDescCreate macros allows for extra optimizations and conveniences
(e.g. IntArrayByRef requires lvalues in C, but can accept rvalues if we use
C++-specific code).

Note that some of the work done in both the C and C++ versions is to provide
type checking. For example, we always want to trigger a warning or error if
the value provided cannot be assigned to a variable of the expected type.
Therefore: in C, we always assign "ctype tempVar = (value)" or
"ctype const* tempVar = (pValue)", and in C++ we always pass (value) or
(pValue) to an appropriately-typed function parameter (e.g. ctype const& or
ctype const*)).
*/

// C++ implementation of DataDescCreate:
#define _tlg_DataDescCreate(n, args) _tlg_ApplyArgs(_tlg_DataDescCreate, args)
#define _tlg_DataDescCreate_tlg_Ignored(...)
#define _tlg_DataDescCreate_tlg_Level(eventLevel)
#define _tlg_DataDescCreate_tlg_Keyword(eventKeyword)
#define _tlg_DataDescCreate_tlg_Int(ctype, value, intFieldType, isSigned, ndt) \
    _tlgCppInit1Desc<ctype>(_tlg_data[_tlg_idx++], (value), _tlg_IntDataSigned##isSigned),
#define _tlg_DataDescCreate_tlg_Enum(ctype, value, etype, intFieldType, isSigned, ndt) \
    _tlgCppInit1Desc<ctype>(_tlg_data[_tlg_idx++], (value), _tlg_IntDataSigned##isSigned),
#define _tlg_DataDescCreate_tlg_IntArrayByRef(ctype, value, cValues, intFieldType, isSigned, ndt) \
    _tlgCppInit1DescByRef<ctype, (cValues)>(_tlg_data[_tlg_idx++], (value)),
#define _tlg_DataDescCreate_tlg_IntArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) \
    _tlgCppInit2DescArray<ctype>(&_tlg_data[_tlg_idx], (pValues), (cValues)),                  \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_IntFixedArray(ctype, pValues, cValues, intFieldType, isSigned, ndt) \
    _tlgCppInit1DescFixedArray<ctype, (cValues)>(_tlg_data[_tlg_idx++], (pValues)),
#define _tlg_DataDescCreate_tlg_Float(ctype, value, ndt) \
    _tlgCppInit1Desc<ctype>(_tlg_data[_tlg_idx++], (value), lttngh_DataType_Float),
#define _tlg_DataDescCreate_tlg_FloatArray(ctype, pValues, cValues, ndt)      \
    _tlgCppInit2DescArray<ctype>(&_tlg_data[_tlg_idx], (pValues), (cValues)), \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_FloatFixedArray(ctype, pValues, cValues, ndt) \
    _tlgCppInit1DescFixedArray<ctype, (cValues)>(_tlg_data[_tlg_idx++], (pValues)),
#define _tlg_DataDescCreate_tlg_Char8(ctype, value, ndt) \
    _tlgCppInit1DescByRef<ctype, 1, ctype>(_tlg_data[_tlg_idx++], (value)),
#define _tlg_DataDescCreate_tlg_CharNN(ctype, value, NN, ndt) \
    _tlgCppInit1DescCharUtf##NN<ctype>(_tlg_data[_tlg_idx++], (value)),
#define _tlg_DataDescCreate_tlg_String8(ctype, pszValue, ndt) \
    _tlgCppInit1DescString8<ctype>(_tlg_data[_tlg_idx++], (pszValue)),
#define _tlg_DataDescCreate_tlg_StringNN(ctype, pszValue, NN, ndt) \
    _tlgCppInit1DescString##NN<ctype>(_tlg_data[_tlg_idx++], (pszValue)),
#define _tlg_DataDescCreate_tlg_CountedString8(ctype, pchValue, cchValue, ndt)  \
    _tlgCppInit2DescArray<ctype>(&_tlg_data[_tlg_idx], (pchValue), (cchValue)), \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_CountedStringNN(ctype, pchValue, cchValue, NN, ndt) \
    _tlgCppInit1DescSeqUtf##NN<ctype>(_tlg_data[_tlg_idx++], (pchValue), (cchValue)),
#define _tlg_DataDescCreate_tlg_Binary(ctype, pValue, cbValue, ndt)                                                   \
    _tlgCppInit2DescArray<ctype, sizeof(uint8_t), lttng_alignof(uint8_t)>(&_tlg_data[_tlg_idx], (pValue), (cbValue)), \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_Sid(ctype, pValue, ndt)         \
    _tlgCppInit2DescSid<ctype>(&_tlg_data[_tlg_idx], (pValue)), \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_GuidPtr(ctype, pValue, name)        \
    _tlgCppInit2DescGuidPtr<ctype>(&_tlg_data[_tlg_idx], (pValue)), \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_Buffer(ctype, pValue, ndt)         \
    _tlgCppInit2DescBuffer<ctype>(&_tlg_data[_tlg_idx], (pValue)), \
        _tlg_idx += 2,
#define _tlg_DataDescCreate_tlg_Value(value, ndt) \
    _tlgCppInit1DescAuto(_tlg_data[_tlg_idx++], (value)),
#else // __cplusplus
// C implementation of DataDescCreate:
#define _tlg_DataDescCreate(n, args) _tlg_ApplyArgsN(_tlg_DataDescCreate, n, args)
#define _tlg_DataDescCreate_tlg_Ignored(n, ...)
#define _tlg_DataDescCreate_tlg_Level(n, eventLevel)
#define _tlg_DataDescCreate_tlg_Keyword(n, eventKeyword)
#define _tlg_DataDescCreate_tlg_Int(n, ctype, value, intFieldType, isSigned, ndt) \
    ctype const _tlg_temp##n = (value);                                           \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreate(&_tlg_temp##n, sizeof(ctype), lttng_alignof(ctype), _tlg_IntDataSigned##isSigned);
#define _tlg_DataDescCreate_tlg_Enum(n, ctype, value, etype, intFieldType, isSigned, ndt) \
    ctype const _tlg_temp##n = (value);                                                   \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreate(&_tlg_temp##n, sizeof(ctype), lttng_alignof(ctype), _tlg_IntDataSigned##isSigned);
#define _tlg_DataDescCreate_tlg_IntArrayByRef(n, ctype, value, cValues, intFieldType, isSigned, ndt) \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateCounted(&(value), (cValues) * sizeof(ctype), lttng_alignof(ctype), (cValues));
#define _tlg_DataDescCreate_tlg_IntArray(n, ctype, pValues, cValues, intFieldType, isSigned, ndt)                 \
    ctype const *const _tlg_temp##n = (pValues);                                                                  \
    _tlg_DataDescCreateArray(&_tlg_data[_tlg_idx], _tlg_temp##n, (cValues), sizeof(ctype), lttng_alignof(ctype)); \
    _tlg_idx += 2;
#define _tlg_DataDescCreate_tlg_IntFixedArray(n, ctype, pValues, cValues, intFieldType, isSigned, ndt) \
    ctype const *const _tlg_temp##n = (pValues);                                                       \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateCounted(_tlg_temp##n, (cValues) * sizeof(ctype), lttng_alignof(ctype), (cValues));
#define _tlg_DataDescCreate_tlg_Float(n, ctype, value, ndt) \
    ctype const _tlg_temp##n = (value);                     \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreate(&_tlg_temp##n, sizeof(ctype), lttng_alignof(ctype), lttngh_DataType_Float);
#define _tlg_DataDescCreate_tlg_FloatArray(n, ctype, pValues, cValues, ndt)                                       \
    ctype const *const _tlg_temp##n = (pValues);                                                                  \
    _tlg_DataDescCreateArray(&_tlg_data[_tlg_idx], _tlg_temp##n, (cValues), sizeof(ctype), lttng_alignof(ctype)); \
    _tlg_idx += 2;
#define _tlg_DataDescCreate_tlg_FloatFixedArray(n, ctype, pValues, cValues, ndt) \
    ctype const *const _tlg_temp##n = (pValues);                                 \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateCounted(_tlg_temp##n, (cValues) * sizeof(ctype), lttng_alignof(ctype), (cValues));
#define _tlg_DataDescCreate_tlg_Char8(n, ctype, value, ndt) \
    ctype const _tlg_temp##n = (value);                     \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateCounted(&_tlg_temp##n, sizeof(ctype), lttng_alignof(ctype), 1);
#define _tlg_DataDescCreate_tlg_CharNN(n, ctype, value, NN, ndt) \
    ctype const _tlg_temp##n = (value);                          \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateSequenceUtf##NN((char##NN##_t const *)&_tlg_temp##n, 1);
#define _tlg_DataDescCreate_tlg_String8(n, ctype, pszValue, ndt) \
    ctype const *const _tlg_temp##n = (pszValue);                \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateString8(_tlg_temp##n ? _tlg_temp##n : "");
#define _tlg_DataDescCreate_tlg_StringNN(n, ctype, pszValue, NN, ndt) \
    ctype const *const _tlg_temp##n = (pszValue);                     \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateStringUtf##NN(_tlg_temp##n ? (char##NN##_t const *)_tlg_temp##n : (char##NN##_t const *)U"");
#define _tlg_DataDescCreate_tlg_CountedString8(n, ctype, pchValue, cchValue, ndt)                                  \
    ctype const *const _tlg_temp##n = (pchValue);                                                                  \
    _tlg_DataDescCreateArray(&_tlg_data[_tlg_idx], _tlg_temp##n, (cchValue), sizeof(ctype), lttng_alignof(ctype)); \
    _tlg_idx += 2;
#define _tlg_DataDescCreate_tlg_CountedStringNN(n, ctype, pchValue, cchValue, NN, ndt) \
    ctype const *const _tlg_temp##n = (pchValue);                                      \
    _tlg_data[_tlg_idx++] = lttngh_DataDescCreateSequenceUtf##NN((char##NN##_t const *)_tlg_temp##n, (cchValue));
#define _tlg_DataDescCreate_tlg_Binary(n, ctype, pValue, cbValue, ndt)                                                \
    ctype const *const _tlg_temp##n = (pValue);                                                                       \
    _tlg_DataDescCreateArray(&_tlg_data[_tlg_idx], _tlg_temp##n, (cbValue), sizeof(uint8_t), lttng_alignof(uint8_t)); \
    _tlg_idx += 2;
#define _tlg_DataDescCreate_tlg_Sid(n, ctype, pValue, ndt)                                                                             \
    ctype const *const _tlg_temp##n = (pValue);                                                                                        \
    _tlg_DataDescCreateArray(&_tlg_data[_tlg_idx], _tlg_temp##n, _tlg_SidSize(_tlg_temp##n), sizeof(uint8_t), lttng_alignof(uint8_t)); \
    _tlg_idx += 2;
#define _tlg_DataDescCreate_tlg_GuidPtr(n, ctype, pValue, name)                                                                       \
    ctype const *const _tlg_temp##n = (pValue);                                                                                       \
    _tlg_DataDescCreateTinyArray(&_tlg_data[_tlg_idx], _tlg_temp##n, _tlg_temp##n ? 16 : 0, sizeof(uint8_t), lttng_alignof(uint8_t)); \
    _tlg_idx += 2;
#define _tlg_DataDescCreate_tlg_Buffer(n, ctype, pValue, ndt)                                                                            \
    ctype const *const _tlg_temp##n = (pValue);                                                                                          \
    _tlg_DataDescCreateArray(&_tlg_data[_tlg_idx], _tlg_temp##n->Buffer, _tlg_temp##n->Length, sizeof(uint8_t), lttng_alignof(uint8_t)); \
    _tlg_idx += 2;
#endif // _cplusplus

#define _tlg_Write_imp(eventProbeFunc, providerSymbol, eventName, ...) ({ \
    static char _tlg_fullName[LTTNG_UST_SYM_NAME_LEN]; /* Filled-in during TraceLoggingRegister. */ \
    static struct lttng_event_field const _tlg_eventFields[0 _tlg_FOREACH(_tlg_FieldCount, __VA_ARGS__)] = { \
        _tlg_FOREACH(_tlg_FieldInfo, __VA_ARGS__) };\
    static struct lttng_ust_tracepoint _tlg_tracepoint \
        __attribute__((section("__tracepoints"))) = { \
        _tlg_fullName, 0, NULL, NULL, "", {} }; \
    static struct lttng_ust_tracepoint* _tlg_tracepointPtr \
        __attribute__((section("__tracepoints_ptrs_" #providerSymbol), used)) = \
        &_tlg_tracepoint; \
    static struct _tlg_Event_t const _tlg_event = { { \
        _tlg_fullName, (void(*)(void))&lttngh_EventProbe, NULL, \
        _tlg_eventFields, sizeof(_tlg_eventFields) / sizeof(struct lttng_event_field), \
        (const int**)&_tlg_event.LevelPtr, "", {} }, \
        (0 _tlg_FOREACH(_tlg_KeywordVal, __VA_ARGS__)), \
        &_tlg_tracepoint, \
        ("" eventName), \
        &_tlg_event.Level, \
        (lttngh_Level_DEBUG _tlg_FOREACH(_tlg_LevelVal, __VA_ARGS__)) }; \
    static struct lttng_event_desc const* _tlg_eventDescPtr \
        __attribute__((section("__eventdesc_ptrs_" #providerSymbol), used)) = \
        &_tlg_event.Desc; \
    int _tlg_err = 0; \
    static_assert(sizeof("" eventName) <= LTTNG_UST_SYM_NAME_LEN - 3, \
        "TraceLoggingWrite eventName must be no more than 253 characters"); \
    (void)&_tlgProv_##providerSymbol; /* Trigger a compile error for misspelled providerSymbol. */ \
    if (caa_unlikely(CMM_LOAD_SHARED(_tlg_tracepoint.state))) { \
        static unsigned const _tlg_idxMax = 0 _tlg_FOREACH(_tlg_DataDescCount, __VA_ARGS__); \
        unsigned _tlg_idx = 0; \
        struct lttngh_DataDesc _tlg_data[_tlg_idxMax]; \
        _tlgBeginCppEval /* For C++, ensure no semicolons until after EventProbe. */ \
        _tlg_FOREACH(_tlg_DataDescCreate, __VA_ARGS__) \
        _tlg_err = eventProbeFunc(&_tlg_tracepoint, _tlg_data, _tlg_idxMax, NULL) \
        _tlgEndCppEval; \
        _tlg_ASSERT(_tlg_idx == _tlg_idxMax); \
    } \
    _tlg_err; })

#ifdef __EDG__
#pragma endregion
#endif

#endif // _TRACELOGGINGPROVIDER_
