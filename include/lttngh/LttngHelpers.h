// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#pragma once
#ifndef _lttnghelpers_h
#define _lttnghelpers_h

#include <lttng/tracepoint-types.h> // lttng_ust_tracepoint
#include <lttng/ust-compiler.h>     // lttng_ust_notrace
#include <lttng/ust-events.h>       // lttng_event, etc.
#include <string.h>                 // strlen
#include <uchar.h>                  // char16_t


// Use lttngh_IP_PARAM to get the address of the caller.
// (For use as the pCallerIp parameter of lttngh_EventProbe.)
#undef lttngh_IP_PARAM
#ifdef TP_IP_PARAM
#define lttngh_IP_PARAM (TP_IP_PARAM)
#elif defined(__PPC__) && !defined(__PPC64__)
#define lttngh_IP_PARAM NULL
#else
#define lttngh_IP_PARAM __builtin_return_address(0)
#endif

enum lttngh_Level {
  lttngh_Level_EMERG = 0,
  lttngh_Level_ALERT = 1,
  lttngh_Level_CRIT = 2,
  lttngh_Level_ERR = 3,
  lttngh_Level_WARNING = 4,
  lttngh_Level_NOTICE = 5,
  lttngh_Level_INFO = 6,
  lttngh_Level_DEBUG_SYSTEM = 7,
  lttngh_Level_DEBUG_PROGRAM = 8,
  lttngh_Level_DEBUG_PROCESS = 9,
  lttngh_Level_DEBUG_MODULE = 10,
  lttngh_Level_DEBUG_UNIT = 11,
  lttngh_Level_DEBUG_FUNCTION = 12,
  lttngh_Level_DEBUG_LINE = 13,
  lttngh_Level_DEBUG = 14
};

// Level values from tracepoint.h:
#define TRACE_EMERG lttngh_Level_EMERG
#define TRACE_ALERT lttngh_Level_ALERT
#define TRACE_CRIT lttngh_Level_CRIT
#define TRACE_ERR lttngh_Level_ERR
#define TRACE_WARNING lttngh_Level_WARNING
#define TRACE_NOTICE lttngh_Level_NOTICE
#define TRACE_INFO lttngh_Level_INFO
#define TRACE_DEBUG_SYSTEM lttngh_Level_DEBUG_SYSTEM
#define TRACE_DEBUG_PROGRAM lttngh_Level_DEBUG_PROGRAM
#define TRACE_DEBUG_PROCESS lttngh_Level_DEBUG_PROCESS
#define TRACE_DEBUG_MODULE lttngh_Level_DEBUG_MODULE
#define TRACE_DEBUG_UNIT lttngh_Level_DEBUG_UNIT
#define TRACE_DEBUG_FUNCTION lttngh_Level_DEBUG_FUNCTION
#define TRACE_DEBUG_LINE lttngh_Level_DEBUG_LINE
#define TRACE_DEBUG lttngh_Level_DEBUG

// Windows trace compatibility macros (adapted from Win32\evntrace.h):
#define TRACE_LEVEL_CRITICAL lttngh_Level_CRIT // Abnormal exit or termination
#define TRACE_LEVEL_FATAL                                                      \
  lttngh_Level_CRIT // Deprecated name for Abnormal exit or termination
#define TRACE_LEVEL_ERROR lttngh_Level_ERR // Severe errors that need logging
#define TRACE_LEVEL_WARNING                                                    \
  lttngh_Level_WARNING // Warnings such as allocation failure
#define TRACE_LEVEL_INFORMATION                                                \
  lttngh_Level_INFO // Includes non-error cases(e.g. Entry-Exit)
#define TRACE_LEVEL_VERBOSE                                                    \
  lttngh_Level_DEBUG                // Detailed traces from intermediate steps
#define EVENT_TRACE_TYPE_INFO 0x00  // Info or point event
#define EVENT_TRACE_TYPE_START 0x01 // Start event
#define EVENT_TRACE_TYPE_END 0x02   // End event
#define EVENT_TRACE_TYPE_STOP 0x02  // Stop event (WinEvent compatible)
#define EVENT_TRACE_TYPE_DC_START 0x03    // Collection start marker
#define EVENT_TRACE_TYPE_DC_END 0x04      // Collection end marker
#define EVENT_TRACE_TYPE_EXTENSION 0x05   // Extension/continuation
#define EVENT_TRACE_TYPE_REPLY 0x06       // Reply event
#define EVENT_TRACE_TYPE_DEQUEUE 0x07     // De-queue event
#define EVENT_TRACE_TYPE_RESUME 0x07      // Resume event (WinEvent compatible)
#define EVENT_TRACE_TYPE_CHECKPOINT 0x08  // Generic checkpoint event
#define EVENT_TRACE_TYPE_SUSPEND 0x08     // Suspend event (WinEvent compatible)
#define EVENT_TRACE_TYPE_WINEVT_SEND 0x09 // Send Event (WinEvent compatible)
#define EVENT_TRACE_TYPE_WINEVT_RECEIVE                                        \
  0XF0 // Receive Event (WinEvent compatible)

// Windows event compatibility macros (adapted from Win32\winmeta.h):
#define WINEVENT_LEVEL_LOG_ALWAYS lttngh_Level_EMERG
#define WINEVENT_LEVEL_CRITICAL lttngh_Level_CRIT
#define WINEVENT_LEVEL_ERROR lttngh_Level_ERR
#define WINEVENT_LEVEL_WARNING lttngh_Level_WARNING
#define WINEVENT_LEVEL_INFO lttngh_Level_NOTICE
#define WINEVENT_LEVEL_VERBOSE lttngh_Level_DEBUG
#define WINEVT_KEYWORD_ANY 0x0
#define WINEVENT_OPCODE_INFO 0x00      // Info or point event
#define WINEVENT_OPCODE_START 0x01     // Start event
#define WINEVENT_OPCODE_STOP 0x02      // Stop event
#define WINEVENT_OPCODE_DC_START 0x03  // Collection start marker
#define WINEVENT_OPCODE_DC_STOP 0x04   // Collection end marker
#define WINEVENT_OPCODE_EXTENSION 0x05 // Extension/continuation
#define WINEVENT_OPCODE_REPLY 0x06     // Reply event
#define WINEVENT_OPCODE_RESUME 0x07    // Resume event
#define WINEVENT_OPCODE_SUSPEND 0x08   // Suspend event
#define WINEVENT_OPCODE_SEND 0x09      // Send Event
#define WINEVENT_OPCODE_RECEIVE 0XF0   // Receive Event

/*
Type of data provided in a lttngh_DataDesc.
*/
enum lttngh_DataType {

  // Payload data that is not exposed to the bytecode filter, e.g.:
  // - The length data for an atype_sequence field.
  // - The content data for atype_dynamic or atype_struct fields.
  lttngh_DataType_None,

  // Signed integer (little-endian): atype_integer, atype_enum.
  lttngh_DataType_SignedLE,

  // Signed integer (big-endian): atype_integer, atype_enum.
  lttngh_DataType_SignedBE,

  // Unsigned integer (little-endian): atype_integer, atype_enum.
  lttngh_DataType_UnsignedLE,

  // Unsigned integer (big-endian): atype_integer, atype_enum.
  lttngh_DataType_UnsignedBE,

  // Floating-point (little-endian): atype_float.
  lttngh_DataType_FloatLE,

  // Floating-point (big-endian): atype_float.
  lttngh_DataType_FloatBE,

  // NUL-terminated string of 8-bit chars: atype_string.
  lttngh_DataType_String8,

  // Arrays and sequences: atype_array, atype_sequence.
  // - For atype_array, the array length (number of elements) is specified
  //   in the field's lttng_type.u.array.length value and must also be
  //   provided in the DataDesc.Length value.
  // - For atype_sequence, a sequence is expressed by two DataDesc items.
  //   The first DataDesc is created via DataDescCreate with Type = None and
  //   contains the sequence's length (number of elements). The second
  //   DataDesc is created via DataDescCreateCounted and contains the data.
  lttngh_DataType_Counted,

  // UTF-16 (host-endian) NUL-terminated string that will be transcoded into
  // a UTF-8 string before storage into the log. The field's lttng_type must
  // be defined as atype_string with UTF8 encoding.
  // Note that when Type = StringUtf16Transcoded, lttngh_EventProbe will
  // use the DataDesc's Length field as scratch space during transcoding.
  lttngh_DataType_StringUtf16Transcoded,

  // UTF-16 (host-endian) string data that will be transcoded into a UTF-8
  // sequence before storage into the log. Unlike a normal sequence (where
  // length is given in a DataDesc with Type = None and content is given in
  // a separate DataDesc with Type = Counted), this DataType requires a
  // single DataDesc, which will be transcoded into payload corresponding to
  // a UTF-8 sequence (including both the length and content). The field's
  // lttng_type must be defined as follows:
  // - atype = sequence.
  // - u.sequence.length_type = uint16, host-endian.
  // - u.sequence.elem_type = uint8, UTF8 encoding.
  // Note that when Type = SequenceUtf16Transcoded, lttngh_EventProbe will
  // use the DataDesc's Length field as scratch space during transcoding.
  lttngh_DataType_SequenceUtf16Transcoded,

  // UTF-32 (host-endian) NUL-terminated string that will be transcoded into
  // a UTF-8 string before storage into the log. The field's lttng_type must
  // be defined as atype_string with UTF8 encoding.
  // Note that when Type = StringUtf32Transcoded, lttngh_EventProbe will
  // use the DataDesc's Length field as scratch space during transcoding.
  lttngh_DataType_StringUtf32Transcoded,

  // UTF-32 (host-endian) string data that will be transcoded into a UTF-8
  // sequence before storage into the log. Unlike a normal sequence (where
  // length is given in a DataDesc with Type = None and content is given in
  // a separate DataDesc with Type = Counted), this DataType requires a
  // single DataDesc, which will be transcoded into payload corresponding to
  // a UTF-8 sequence (including both the length and content). The field's
  // lttng_type must be defined as follows:
  // - atype = sequence.
  // - u.sequence.length_type = uint16, host-endian.
  // - u.sequence.elem_type = uint8, UTF8 encoding.
  // Note that when Type = SequenceUtf32Transcoded, lttngh_EventProbe will
  // use the DataDesc's Length field as scratch space during transcoding.
  lttngh_DataType_SequenceUtf32Transcoded,

  // Signed integer (host-endian): atype_integer, atype_enum.
  lttngh_DataType_Signed = BYTE_ORDER == LITTLE_ENDIAN
                               ? lttngh_DataType_SignedLE
                               : lttngh_DataType_SignedBE,

  // Unsigned integer (host-endian): atype_integer, atype_enum.
  lttngh_DataType_Unsigned = BYTE_ORDER == LITTLE_ENDIAN
                                 ? lttngh_DataType_UnsignedLE
                                 : lttngh_DataType_UnsignedBE,

  // Floating-point (host-endian): atype_float.
  lttngh_DataType_Float = FLOAT_WORD_ORDER == LITTLE_ENDIAN
                              ? lttngh_DataType_FloatLE
                              : lttngh_DataType_FloatBE
};

/*
The payload for an event is described by an array of DataDesc objects.
Use a DataDescCreate helper to fill in a DataDesc object.

In general, the event payload is simply the concatenation of the data bytes
described by Data and Size, with padding based on Alignment. Exceptions
to this rule (i.e. the cases where Type is used):
- If Type is String8, the string may be padded with '#' if the string's actual
  length is smaller than the length indicated by the Size field.
- If Type is SequenceUtf16Transcoded or SequenceUtf32Transcoded, the event
  payload will be a transcoding of Data, not a copy of Data.

Length is only used by the bytecode filter.
*/
struct lttngh_DataDesc {
  void const *Data;
  uint32_t Size;     // = sizeof(element) * number of elements
  uint8_t Alignment; // = lttng_alignof(element)

  // The following fields are mainly for bytecode filtering:
  uint8_t Type;    // Use values from enum lttngh_DataType.
  uint16_t Length; // = number of elements; only used if Type == Counted.
};

#if (LTTNG_UST_MAJOR_VERSION < 2 ||                                            \
     (LTTNG_UST_MAJOR_VERSION == 2 && LTTNG_UST_MINOR_VERSION <= 6))
#error "Please use LTTNG 2.7+"
#elif (LTTNG_UST_MAJOR_VERSION == 2 && LTTNG_UST_MINOR_VERSION == 7)
#define USE_LTTNG_2_7
#endif

#ifdef USE_LTTNG_2_7
#define LTTNG_ENUM_TYPE lttng_enum
#else
#define LTTNG_ENUM_TYPE lttng_enum_desc
#endif

#ifdef __cplusplus
extern "C" {
#endif

/*
Predefined enumeration descriptor for use with bool/BOOL/BOOLEAN.
*/
extern struct LTTNG_ENUM_TYPE const lttngh_BoolEnumDesc;

/*
Registers a provider. The pIsRegistered pointer will be atomically updated
with registration state.

LTTNG tracks providers by name, and does not support registering multiple
providers with the same name within a process. If this is attempted,
RegisterProvider will return an error code.

It is an error to invoke RegisterProvider when the provider is already
registered (as tracked by pIsRegistered). If this happens,
RegisterProvider will call abort().
*/
int lttngh_RegisterProvider(
    int volatile *pIsRegistered, struct lttng_probe_desc *pProbeDesc,
    struct lttng_ust_tracepoint **pTracepointStart,
    struct lttng_ust_tracepoint **pTracepointStop,
    struct lttng_event_desc const **pEventDescStart,
    struct lttng_event_desc const **pEventDescStop) lttng_ust_notrace;

/*
Unregisters a provider. The pIsRegistered pointer will be atomically
updated with registration state.

It is not an error to invoke UnregisterProvider when the provider is already
unregistered (as tracked by pIsRegistered). If this happens,
UnregisterProvider is a no-op and immediately returns.
*/
int lttngh_UnregisterProvider(
    int volatile *pIsRegistered, struct lttng_probe_desc *pProbeDesc,
    struct lttng_ust_tracepoint *const *pTracepointStart) lttng_ust_notrace;

/*
Writes an event with the data from the array of DataDesc objects.
If pCallerIp is NULL, EventProbe will use the immediate caller's address.
Note: lttngh_EventProbe may use pDataDesc[n].Length as scratch space,
which is why pDataDesc is not defined as "const".
*/
int lttngh_EventProbe(struct lttng_ust_tracepoint *pTracepoint,
                      struct lttngh_DataDesc *pDataDesc, unsigned cDataDesc,
                      void *pCallerIp) // usually NULL or lttngh_IP_PARAM
    __attribute__((noinline)) lttng_ust_notrace;

/*
For use by code that is starting an activity.
Generates a new 16-byte activity ID and copies it to pActivityId.
Note that while the generated ID has the structure of a GUID, it is not
guaranteed to be globally-unique. Instead, activity ID generation is
optimized for speed. The generated ID is intended to be locally-unique
(within the current boot session) but may not be globally unique.

Activity types:
- Manually-controlled activities: Activity lifetime can flow across threads
  (or even across processes) as needed, but activity ID must be tracked by
  the developer and directly specified for each event.
- Thread-local (scoped) activities: Activity ID is automatically applied to
  each event (via a thread-local variable), but activity lifetime must
  correspond to a local scope on a single thread, e.g. activity starts at
  function entry and ends at function exit.
  Note that the previous activity ID should be restored at scope exit even
  if the exit is abnormal, i.e. even if scope exit occurs via C++ exception
  (otherwise you're clobbering another scope's activity ID with your own).

Usage pattern for manually-controlled activities:
- Determine the value of parentId, which is the ID of the higher-level
  activity (or NULL if no higher-level activity).
- Call lttngh_ActivityIdCreate to obtain a value for newId.
- Write an activity-start event using Opcode=START, ActivityId=newId, and
  RelatedActivityId=parentId.
- Write info events using ActivityId=newId.
- Write an activity-stop event using Opcode=STOP, ActivityId=newId.

Usage pattern for thread-local (scoped) activities:
- The following pattern assumes that your event generation system (the
  layer between the user and the lttngh_EventProbe function) uses the
  lttngh_ActivityIdFilter function to obtain the activity ID value to use
  when no activity ID is provided by the user.
- Call lttngh_ActivityIdGet to obtain the value of parentId.
- Call lttngh_ActivityIdCreate to obtain a value for newId.
- Call lttngh_ActivityIdSet to change the current thread's ID to newId.
- Write an activity-start event using Opcode=START, ActivityId=NULL, and
  RelatedActivityId=parentId.
- Write info events.
- Write an activity-end event using Opcode=STOP.
- Call lttngh_ActivityIdSet to restore the current thread's ID to parentId.
*/
void lttngh_ActivityIdCreate(void *pNewActivityId) lttng_ust_notrace;

/*
For use by code that is starting a thread-local activity.
Copies the current thread-local 16-byte activity ID to
pCurrentThreadActivityId. Note that when a new thread is created, its
activity ID is initialized to all-zero.

To start a thread-local activity:
- Call lttngh_ActivityIdGet to obtain the value of parentId.
- Call lttngh_ActivityIdCreate to obtain a value for newId.
- Call lttngh_ActivityIdSet to change the current thread's ID to newId.
- Write an activity-start event using Opcode=START, ActivityId=NULL, and
  RelatedActivityId=parentId.
*/
void lttngh_ActivityIdGet(void *pCurrentThreadActivityId) lttng_ust_notrace;

/*
For use by code that is starting or ending a thread-local activity.
Copies the specified 16-byte activity ID to the current thread-local
activity ID.

To start a thread-local activity:
- Call lttngh_ActivityIdGet to obtain the value of parentId.
- Call lttngh_ActivityIdCreate to obtain a value for newId.
- Call lttngh_ActivityIdSet to change the current thread's ID to newId.
- Write an activity-start event using Opcode=START, ActivityId=NULL, and
  RelatedActivityId=parentId.

To end a thread-local activity:
- Write an activity-end event using Opcode=STOP.
- Call lttngh_ActivityIdSet to restore the current thread's ID to parentId.
*/
void lttngh_ActivityIdSet(void const *pNewThreadActivityId) lttng_ust_notrace;

/*
Implementation detail of lttngh_ActivityIdFilter. Do not call directly.
- If current thread-local activity ID is non-zero, return a pointer to the
  current thread-local activity ID.
- Otherwise, return NULL.
*/
void const *lttngh_ActivityIdPeek(void) lttng_ust_notrace;

/*
For use by code that is generating events.
Returns a pointer to the activity ID that should be used for an event.
- If pUserProvidedActivityId != NULL, return pUserProvidedActivityId.
- Otherwise, if current thread-local activity ID is non-zero, return a
  pointer to the current thread-local activity ID.
- Otherwise, return NULL.
*/
static inline void const *
lttngh_ActivityIdFilter(void const *pUserProvidedActivityId) lttng_ust_notrace;
static inline void const *
lttngh_ActivityIdFilter(void const *pUserProvidedActivityId) {
  return pUserProvidedActivityId ? pUserProvidedActivityId
                                 : lttngh_ActivityIdPeek();
}

/*
Constructs a DataDesc object for scalar data.
Use this for types other than array or sequence.
*/
static inline struct lttngh_DataDesc lttngh_DataDescCreate(
    const void *data,          // = &value
    unsigned size,             // = sizeof(value)
    unsigned char alignment,   // = lttng_alignof(value)
    enum lttngh_DataType type) // = None, Signed, Unsigned, Float
    lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreate(const void *data, unsigned size, unsigned char alignment,
                      enum lttngh_DataType type) {
  assert(
      type == lttngh_DataType_None || type == lttngh_DataType_SignedLE ||
      type == lttngh_DataType_SignedBE || type == lttngh_DataType_UnsignedLE ||
      type == lttngh_DataType_UnsignedBE || type == lttngh_DataType_FloatLE ||
      type == lttngh_DataType_FloatBE || type == lttngh_DataType_String8);
  struct lttngh_DataDesc dd = {data, size, alignment, (uint8_t)type, 0};
  return dd;
}

/*
Constructs a DataDesc object for a nul-terminated string of 8-bit chars.
The returned DataDesc will have Type = String8.
Note that str must not be NULL and must be NUL-terminated.
*/
static inline struct lttngh_DataDesc
lttngh_DataDescCreateString8(const char *str) lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateString8(const char *str) {
  struct lttngh_DataDesc dd = {str, (unsigned)strlen(str) + 1,
                               lttng_alignof(str[0]), lttngh_DataType_String8,
                               0};
  return dd;
}

/*
Constructs a DataDesc object for the data of an array or sequence.
The returned DataDesc will have Type = Counted.
*/
static inline struct lttngh_DataDesc lttngh_DataDescCreateCounted(
    const void *data,        // = &value
    unsigned size,           // = sizeof(element) * length
    unsigned char alignment, // = lttng_alignof(element)
    unsigned length)         // = Number of elements in value
    lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateCounted(const void *data, unsigned size,
                             unsigned char alignment, unsigned length) {
  struct lttngh_DataDesc dd = {
      data, size, alignment, lttngh_DataType_Counted,
      // Length is only used by bytecode filters. Truncation probably ok.
      (uint16_t)(length > 65535u ? 65535u : length)};
  return dd;
}

/*
Constructs a DataDesc object for a nul-terminated string of UTF-16 chars.
The returned DataDesc will have Type = StringUtf16Transcoded.
Note that str must not be NULL and must be NUL-terminated.
*/
static inline struct lttngh_DataDesc
lttngh_DataDescCreateStringUtf16(const char16_t *str) lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateStringUtf16(const char16_t *str) {
  const char16_t *strEnd = str;
  while (*strEnd++ != 0) {
  }
  struct lttngh_DataDesc dd = {
      str, (unsigned)(strEnd - str) * (unsigned)sizeof(char16_t),
      lttng_alignof(char16_t), lttngh_DataType_StringUtf16Transcoded, 0};
  return dd;
}

/*
Constructs a DataDesc object for a counted string of UTF-16 chars.
The returned DataDesc will have Type = SequenceUtf16Transcoded.
Note that str may be NULL only if length is 0.
*/
static inline struct lttngh_DataDesc lttngh_DataDescCreateSequenceUtf16(
    const char16_t *str, // = &char_array
    uint16_t length)     // = Number of code points in string
    lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateSequenceUtf16(const char16_t *str, uint16_t length) {
  struct lttngh_DataDesc dd = {str, length * (unsigned)sizeof(char16_t),
                               lttng_alignof(char16_t),
                               lttngh_DataType_SequenceUtf16Transcoded, 0};
  return dd;
}

/*
Constructs a DataDesc object for a nul-terminated string of UTF-32 chars.
The returned DataDesc will have Type = StringUtf32Transcoded.
Note that str must not be NULL and must be NUL-terminated.
*/
static inline struct lttngh_DataDesc
lttngh_DataDescCreateStringUtf32(const char32_t *str) lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateStringUtf32(const char32_t *str) {
  const char32_t *strEnd = str;
  while (*strEnd++ != 0) {
  }
  struct lttngh_DataDesc dd = {
      str, (unsigned)(strEnd - str) * (unsigned)sizeof(char32_t),
      lttng_alignof(char32_t), lttngh_DataType_StringUtf32Transcoded, 0};
  return dd;
}

/*
Constructs a DataDesc object for a counted string of UTF-32 chars.
The returned DataDesc will have Type = SequenceUtf32Transcoded.
Note that str may be NULL only if length is 0.
*/
static inline struct lttngh_DataDesc lttngh_DataDescCreateSequenceUtf32(
    const char32_t *str, // = &char_array
    uint16_t length)     // = Number of code points in string
    lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateSequenceUtf32(const char32_t *str, uint16_t length) {
  struct lttngh_DataDesc dd = {str, length * (unsigned)sizeof(char32_t),
                               lttng_alignof(char32_t),
                               lttngh_DataType_SequenceUtf32Transcoded, 0};
  return dd;
}

/*
Constructs a DataDesc object for a nul-terminated string of wchar_t chars.
The returned DataDesc will have Type = SequenceUtf16Transcoded or
SequenceUtf32Transcoded depending on the encoding used by wchar_t.
Note that str must not be NULL and must be NUL-terminated.
*/
static inline struct lttngh_DataDesc
lttngh_DataDescCreateStringWchar(const wchar_t *str) lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateStringWchar(const wchar_t *str) {
#if (__WCHAR_MAX == 0x7fffffff) || (__WCHAR_MAX == 0xffffffff)
  return lttngh_DataDescCreateStringUtf32((char32_t const *)str);
#elif (__WCHAR_MAX == 0x7fff) || (__WCHAR_MAX == 0xffff)
  return lttngh_DataDescCreateStringUtf16((char16_t const *)str);
#else
#error Unsupported wchar_t type.
#endif // __WCHAR_MAX
}

/*
Constructs a DataDesc object for a counted string of wchar_t chars.
The returned DataDesc will have Type = SequenceUtf16Transcoded or
SequenceUtf32Transcoded depending on the encoding used by wchar_t.
Note that str may be NULL only if length is 0.
*/
static inline struct lttngh_DataDesc lttngh_DataDescCreateSequenceWchar(
    const wchar_t *str, // = &char_array
    uint16_t length)    // = Number of code points in string
    lttng_ust_notrace;
static inline struct lttngh_DataDesc
lttngh_DataDescCreateSequenceWchar(const wchar_t *str, uint16_t length) {
#if (__WCHAR_MAX == 0x7fffffff) || (__WCHAR_MAX == 0xffffffff)
  return lttngh_DataDescCreateSequenceUtf32((char32_t const *)str, length);
#elif (__WCHAR_MAX == 0x7fff) || (__WCHAR_MAX == 0xffff)
  return lttngh_DataDescCreateSequenceUtf16((char16_t const *)str, length);
#else
#error Unsupported wchar_t type.
#endif // __WCHAR_MAX
}

#ifdef __cplusplus
} // extern "C"
#endif

#endif // _lttnghelpers_h
