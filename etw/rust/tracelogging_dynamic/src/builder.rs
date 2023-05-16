// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::copy_nonoverlapping;

use tracelogging::Channel;
use tracelogging::Guid;
use tracelogging::InType;
use tracelogging::Level;
use tracelogging::Opcode;
use tracelogging::OutType;
use tracelogging::_internal::EventDataDescriptor;
use tracelogging::_internal::EventDescriptor;

use crate::provider::Provider;

/// `EventBuilder` is a builder for events to be written through a [Provider].
///
/// # Overview
///
/// - Check [Provider::enabled], e.g. `my_provider.enabled(level, keyword)`, so that you
///   skip the remaining steps if no ETW logging sessions are listening for your event.
/// - Get an EventBuilder, e.g. `let mut builder = EventBuilder::new();`.
///   - EventBuilder is reusable. You may get a small performance improvement by reusing
///     builders instead of creating a new one for each event.
/// - Call `builder.reset("EventName", level, keyword, event_tag)` to begin building an event.
///   - The event name should be short and distinct. Don't use same name for two events in
///     the same provider that have different fields, levels, or keywords.
///   - level is the event severity.
///   - keyword is a bitmask indicating one or more event categories.
///   - event_tag is a 28-bit provider-defined value that will be included in the
///     metadata of the event. Use 0 if you are not using event tags.
/// - For each field of the event, call
///   `builder.add_TYPE("FieldName", field_value, OutType::OUTTYPE, field_tag)`.
///   - The method's TYPE suffix maps to an [InType] that specifies the encoding of the
///     field as well as the default formatting that should apply if [OutType::Default]
///     is used. For example, `add_hex32` maps to [InType::Hex32].
///   - The field name should be short and distinct.
///   - The OUTTYPE controls the formatting that will be used when the field is decoded.
///     Use [OutType::Default] to get the normal formatting based on the method's TYPE
///     suffix (i.e. the default format for the corresponding [InType]). Use other
///     [OutType] values when non-default formatting is needed. For example, `add_u8`
///     adds an [InType::U8] field to the event. The default format for [InType::U8] is
///     [OutType::Unsigned], so if you use `add_u8(..., OutType::Default, ...), the field
///     will be decoded as an unsigned decimal integer. However, if you specify
///     [OutType::String], the field will be formatted as a char, or if you specify
///     [OutType::Hex], the field will be formatted as a hexadecimal integer.
///     - Note that [OutType::Default] has a special encoding that saves 1 byte per
///       field, so prefer [OutType::Default] over other [OutType] values in cases where
///       they both do the same thing.
///   - field_tag is a 28-bit provider-defined value that will be included in the
///     metadata of the field. Use 0 if you are not using field tags.
/// - If appropriate, configure other event options by calling:
///   - `builder.id_version(id, ver)`
///   - `builder.channel(channel)`
///   - `builder.opcode(opcode)`
///   - `builder.task(task)`
/// - Call `builder.write(provider, activity_id, related_id)` to
///   send the event to ETW.
///   - `activity_id` is an optional 128-bit value that can be used during trace
///     analysis to group and correlate events. If `None`, the current thread's
///     thread-local activity id will be used as the event's activity id.
///   - `related_id` is an optional 128-bit value that indicates the parent of a
///     newly-started activity. This should be specified for
///     activity-[start](Opcode::ActivityStart) events and should be `None` for other events.
///
/// # Event Size Limits
///
/// ETW does not support events larger than the event consumer's buffer size. In
/// addition, regardless of the event consumer's buffer size, ETW does not support events
/// larger than 64KB. The event size includes event headers (provider id, activity id,
/// timestamp, event descriptor, etc.), metadata (provider name, event name, field names),
/// and data (field values). Events that are too large will cause builder.write to return
/// an error.
///
/// Most ETW decoding tools are unable to decode an event with more than 128 fields.
#[derive(Debug)]
pub struct EventBuilder {
    meta: Vec<u8>,
    data: Vec<u8>,
    descriptor: EventDescriptor,
}

impl EventBuilder {
    /// Returns a new event builder with default initial buffer capacity.
    ///
    /// Default capacity is currently 256 bytes for meta and 256 bytes for data.
    /// Buffers will automatically grow as needed.
    pub fn new() -> EventBuilder {
        return Self::new_with_capacity(256, 256);
    }

    /// Returns a new event builder with specified initial buffer capacities.
    /// Buffers will automatically grow as needed.
    pub fn new_with_capacity(meta_capacity: u16, data_capacity: u16) -> EventBuilder {
        let mut b = EventBuilder {
            meta: Vec::with_capacity(if meta_capacity < 4 {
                4
            } else {
                meta_capacity as usize
            }),
            data: Vec::with_capacity(data_capacity as usize),
            descriptor: EventDescriptor::zero(),
        };
        b.meta.resize(4, 0); // u16 size = 0, u8 tag = 0, u8 name_nul_termination = 0;
        return b;
    }

    /// Clears the previous event (if any) from the builder and starts building a new
    /// event.
    ///
    /// name is the event name. It should be short and unique. It must not contain any
    /// `'\0'` bytes.
    ///
    /// level indicates the severity of the event. Use Verbose if unsure.
    ///
    /// keyword is a bitmask of category bits. The upper 16 bits are defined by Microsoft,
    /// and the low 48 bits are defined by the user on a per-provider basis, i.e. all
    /// providers with a particular name + id should use the same category bit assignments.
    /// For example, your provider might define 0x2 as the "networking" category bit and 0x4
    /// as the "threading" category bit. Keyword should always be a non-zero value. If
    /// category bits have not been defined for your provider, define 0x1 as "uncategorized"
    /// and use 0x1 as the keyword for all events.
    ///
    /// event_tag is a 28-bit integer (range 0x0 to 0x0FFFFFFF). Use 0 if you are
    /// not using event tags.
    pub fn reset(&mut self, name: &str, level: Level, keyword: u64, event_tag: u32) -> &mut Self {
        debug_assert!(!name.contains('\0'), "event name must not contain '\\0'");
        debug_assert_eq!(
            event_tag & 0x0FFFFFFF,
            event_tag,
            "event_tag must fit into 28 bits"
        );

        self.meta.clear();
        self.data.clear();
        self.descriptor = EventDescriptor::new(level, keyword);

        // Placeholder for u16 metadata size, filled-in by write.
        self.meta.push(0);
        self.meta.push(0);

        if (event_tag & 0x0FE00000) == event_tag {
            self.meta.push((event_tag >> 21) as u8);
        } else if (event_tag & 0x0FFFC000) == event_tag {
            self.meta.push((event_tag >> 21) as u8 | 0x80);
            self.meta.push((event_tag >> 14) as u8 & 0x7F);
        } else {
            self.meta.push((event_tag >> 21) as u8 | 0x80);
            self.meta.push((event_tag >> 14) as u8 | 0x80);
            self.meta.push((event_tag >> 7) as u8 | 0x80);
            self.meta.push(event_tag as u8 & 0x7F);
        }

        self.meta.extend_from_slice(name.as_bytes());
        self.meta.push(0); // nul termination

        return self;
    }

    /// Sends the built event to ETW via the specified provider.
    ///
    /// Returns 0 for success or a Win32 error from `EventWrite` for failure. The return
    /// value is for diagnostic purposes only and should generally be ignored in retail
    /// builds.
    ///
    /// provider: Should usually be a registered provider. Calling write on an
    /// unregistered provider is a no-op.
    ///
    /// activity_id: Contains the activity id to be used for the event. If None, the event will
    /// use the current thread's thread-local activity id for its activity id.
    ///
    /// related_id: Contains the related activity id (parent activity) to be used for the event.
    /// If None, the event will not have a related activity id. The related activity id should
    /// be set for activity-start events and should be None for other events.
    pub fn write(
        &mut self,
        provider: &Provider,
        activity_id: Option<&Guid>,
        related_id: Option<&Guid>,
    ) -> u32 {
        let result;
        let meta_len = self.meta.len();
        if meta_len > 65535 {
            result = 534; // ERROR_ARITHMETIC_OVERFLOW
        } else {
            self.meta[0] = meta_len as u8;
            self.meta[1] = (meta_len >> 8) as u8;
            let dd = [
                EventDataDescriptor::from_raw_bytes(&provider.meta, 2), // EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA
                EventDataDescriptor::from_raw_bytes(&self.meta, 1), // EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA
                EventDataDescriptor::from_raw_bytes(&self.data, 0), // EVENT_DATA_DESCRIPTOR_TYPE_NONE
            ];
            let ctx = &provider.context;
            result = ctx.write_transfer(
                &self.descriptor,
                activity_id.map(|g| g.as_bytes_raw()),
                related_id.map(|g| g.as_bytes_raw()),
                &dd,
            );
        }
        return result;
    }

    /// Sets the id and version of the event. Default is id = 0, version = 0.
    ///
    /// TraceLogging events are primarily identified by event name, not by id.
    /// Most events use id = 0, version = 0 and therefore do not need to call this
    /// method.
    ///
    /// Events should use id = 0 and version = 0 unless they have a manually-assigned
    /// stable id. If the event has a manually-assigned stable id, it must be a nonzero
    /// value and the version should be incremented each time the event schema changes
    /// (i.e. each time the field names or field types are changed).
    pub fn id_version(&mut self, id: u16, version: u8) -> &mut Self {
        self.descriptor.id = id;
        self.descriptor.version = version;
        return self;
    }

    /// Sets the channel of the event. Default channel = TraceLogging.
    ///
    /// Most events should use channel = TraceLogging and therefore do not need to call
    /// this method.
    pub fn channel(&mut self, channel: Channel) -> &mut Self {
        self.descriptor.channel = channel;
        return self;
    }

    /// Sets the opcode of the event. Default opcode = Info.
    ///
    /// Most events use opcode = Info and therefore do not need to call this method.
    ///
    /// You can use opcode to create an activity (a group of related events):
    ///
    /// 1. Begin the activity by writing an activity-start event with opcode = Start,
    ///    activity_id = the id of the new activity (generated by [Guid::new] or
    ///    [Provider::create_activity_id]), and related_id = the id of the parent
    ///    activity.
    /// 2. As appropriate, write activity-info events with opcode = Info,
    ///    activity_id = the id of the activity, and related_id = None.
    /// 3. End the activity by writing an activity-stop event with opcode = Stop,
    ///    activity_id = the id of the activity, related_id = None, and the same level
    ///    and keyword as were used for the activity-start event.
    pub fn opcode(&mut self, opcode: Opcode) -> &mut Self {
        self.descriptor.opcode = opcode;
        return self;
    }

    /// Sets the task of the event. Default task = 0.
    ///
    /// Most events use task = 0 and therefore do not need to call this method.
    ///
    /// Task is a provider-defined 16-bit value assigned to the event. It can be used
    /// for any purpose, but is typically used as an abstract event id, e.g. while there
    /// might be 2 or 3 distinct events that indicate a different type of network packet
    /// was written, all of them could be assigned the same nonzero "packet written"
    /// task identifier.
    pub fn task(&mut self, task: u16) -> &mut Self {
        self.descriptor.task = task;
        return self;
    }

    /// Adds a CStr16 field (nul-terminated UTF16-LE) from a `&[u16]` value.
    ///
    /// If the string contains characters after a `'\0'`, they will be discarded.
    /// If the string contains no `'\0'` chars, one will be added automatically.
    ///
    /// If out_type is Default, field will format as String.
    /// Other useful out_type values: Xml, Json.
    ///
    /// This is the same as `add_str16` except that the ETW field will be encoded
    /// as a nul-terminated string instead of as a counted string. In most cases
    /// you should prefer `add_str16` and use this method only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_cstr16(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u16]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::CStr16, out_type, field_tag)
            .raw_add_data_cstr(field_value.as_ref());
    }

    /// Adds a CStr16 variable-length array field (nul-terminated UTF16-LE) from an
    /// iterator-of-`&[u16]` value.
    ///
    /// If the string contains characters after a `'\0'`, they will be discarded.
    /// If the string contains no `'\0'` chars, one will be added automatically.
    ///
    /// If out_type is Default, field will format as String.
    /// Other useful out_type values: Xml, Json.
    ///
    /// This is the same as `add_str16_sequence` except that the ETW field will be encoded
    /// as a nul-terminated string instead of as a counted string. In most cases
    /// you should prefer `add_str16_sequence` and use this method only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_cstr16_sequence<T: IntoIterator>(
        &mut self,
        field_name: &str,
        field_values: T,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self
    where
        T::Item: AsRef<[u16]>,
    {
        return self
            .raw_add_meta_vcount(field_name, InType::CStr16, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_cstr(value.as_ref());
            });
    }

    /// Adds a CStr8 field (nul-terminated 8-bit string) from a `&[u8]` value.
    ///
    /// If the string contains characters after a `'\0'`, they will be discarded.
    /// If the string contains no `'\0'` chars, one will be added automatically.
    ///
    /// If out_type is Default, field will format as String (CP1252, not UTF-8).
    /// Other useful out_type values: Xml, Json, Utf8 (all of which decode as UTF-8).
    ///
    /// This is the same as `add_str8` except that the ETW field will be encoded
    /// as a nul-terminated string instead of as a counted string. In most cases
    /// you should prefer `add_str8` and use this method only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_cstr8(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u8]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::CStr8, out_type, field_tag)
            .raw_add_data_cstr(field_value.as_ref());
    }

    /// Adds a CStr8 variable-length array field (nul-terminated 8-bit string) from an
    /// iterator-of-`&[u8]` value.
    ///
    /// If the string contains characters after a `'\0'`, they will be discarded.
    /// If the string contains no `'\0'` chars then one will be added automatically.
    ///
    /// If out_type is Default, field will format as String (CP1252, not UTF-8).
    /// Other useful out_type values: Xml, Json, Utf8 (all of which decode as UTF-8).
    ///
    /// This is the same as `add_str8_sequence` except that the ETW field will be encoded
    /// as a nul-terminated string instead of as a counted string. In most cases
    /// you should prefer `add_str8_sequence` and use this method only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_cstr8_sequence<T: IntoIterator>(
        &mut self,
        field_name: &str,
        field_values: T,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self
    where
        T::Item: AsRef<[u8]>,
    {
        return self
            .raw_add_meta_vcount(field_name, InType::CStr8, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_cstr(value.as_ref());
            });
    }

    /// Adds an I8 field from an `i8` value.
    ///
    /// If out_type is Default, field will format as Signed.
    /// Other useful out_type value: String (formats as CP1252 character).
    pub fn add_i8(
        &mut self,
        field_name: &str,
        field_value: i8,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::I8, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an I8 variable-length array field from an iterator-of-`&i8` value.
    ///
    /// If out_type is Default, field will format as Signed.
    /// Other useful out_type value: String (formats as CP1252 character).
    pub fn add_i8_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a i8>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::I8, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a U8 field from a `u8` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type values: Hex, String (formats as CP1252 char), Boolean.
    pub fn add_u8(
        &mut self,
        field_name: &str,
        field_value: u8,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::U8, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a U8 variable-length array field from an iterator-of-`&u8` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type values: Hex, String (formats as CP1252 char), Boolean.
    pub fn add_u8_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a u8>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::U8, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds an I16 field from an `i16` value.
    ///
    /// If out_type is Default, field will format as Signed.
    pub fn add_i16(
        &mut self,
        field_name: &str,
        field_value: i16,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::I16, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an I16 variable-length array field from an iterator-of-`&i16` value.
    ///
    /// If out_type is Default, field will format as Signed.
    pub fn add_i16_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a i16>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::I16, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a U16 field from a `u16` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type values: Hex, String (formats as UCS-2 char), Port.
    pub fn add_u16(
        &mut self,
        field_name: &str,
        field_value: u16,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::U16, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a U16 variable-length array field from an iterator-of-`&u16` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type values: Hex, String (formats as UCS-2 char), Port.
    pub fn add_u16_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a u16>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::U16, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds an I32 field from an `i32` value.
    ///
    /// If out_type is Default, field will format as Signed.
    /// Other useful out_type value: HResult.
    pub fn add_i32(
        &mut self,
        field_name: &str,
        field_value: i32,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::I32, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an I32 variable-length array field from an iterator-of-`&i32` value.
    ///
    /// If out_type is Default, field will format as Signed.
    /// Other useful out_type value: HResult.
    pub fn add_i32_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a i32>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::I32, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a U32 field from a `u32` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type values: Pid, Tid, IPv4, Win32Error, NtStatus, CodePointer.
    pub fn add_u32(
        &mut self,
        field_name: &str,
        field_value: u32,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::U32, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a U32 variable-length array field from an iterator-of-`&u32` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type values: Pid, Tid, IPv4, Win32Error, NtStatus, CodePointer.
    pub fn add_u32_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a u32>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::U32, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds an I64 field from an `i64` value.
    ///
    /// If out_type is Default, field will format as Signed.
    pub fn add_i64(
        &mut self,
        field_name: &str,
        field_value: i64,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::I64, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an I64 variable-length array field from an iterator-of-`&i64` value.
    ///
    /// If out_type is Default, field will format as Signed.
    pub fn add_i64_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a i64>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::I64, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a U64 field from a `u64` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type value: CodePointer.
    pub fn add_u64(
        &mut self,
        field_name: &str,
        field_value: u64,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::U64, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a U64 variable-length array field from an iterator-of-`&u64` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type value: CodePointer.
    pub fn add_u64_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a u64>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::U64, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds an ISize field from an `isize` value.
    ///
    /// If out_type is Default, field will format as Signed.
    pub fn add_isize(
        &mut self,
        field_name: &str,
        field_value: isize,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::ISize, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an ISize variable-length array field from an iterator-of-`&isize` value.
    ///
    /// If out_type is Default, field will format as Signed.
    pub fn add_isize_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a isize>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::ISize, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a USize field from a `usize` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type value: CodePointer.
    pub fn add_usize(
        &mut self,
        field_name: &str,
        field_value: usize,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::USize, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a USize variable-length array field from an iterator-of-`&usize` value.
    ///
    /// If out_type is Default, field will format as Unsigned.
    /// Other useful out_type value: CodePointer.
    pub fn add_usize_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a usize>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::USize, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds an F32 field from an `f32` value.
    ///
    /// If out_type is Default, field will format as float.
    pub fn add_f32(
        &mut self,
        field_name: &str,
        field_value: f32,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::F32, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an F32 variable-length array field from an iterator-of-`&f32` value.
    ///
    /// If out_type is Default, field will format as float.
    pub fn add_f32_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a f32>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::F32, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds an F64 field from an `f64` value.
    ///
    /// If out_type is Default, field will format as float.
    pub fn add_f64(
        &mut self,
        field_name: &str,
        field_value: f64,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::F64, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds an F64 variable-length array field from an iterator-of-`&f64` value.
    ///
    /// If out_type is Default, field will format as float.
    pub fn add_f64_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a f64>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::F64, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a Bool32 field from an `i32` value.
    ///
    /// If out_type is Default, field will format as Boolean.
    pub fn add_bool32(
        &mut self,
        field_name: &str,
        field_value: i32,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Bool32, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a Bool32 variable-length array field from an iterator-of-`&i32` value.
    ///
    /// If out_type is Default, field will format as Boolean.
    pub fn add_bool32_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a i32>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::Bool32, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a Binary field from a `&[u8]` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: IPv6, SocketAddress, Pkcs7WithTypeInfo.
    ///
    /// This is the same as `add_binaryc` except that it uses an older ETW encoding.
    /// - Older ETW [InType], so decoding works in all versions of Windows.
    /// - Decoded event frequently includes a synthesized "FieldName.Length" field.
    /// - Arrays are not supported.
    ///
    /// Note: There is no `add_binary_sequence` method because the ETW's `Binary` encoding does
    /// not decode correctly with arrays. Array of binary can be created using
    /// `add_binaryc_sequence`, though the resulting event will only decode correctly
    /// if the decoder supports ETW's newer `BinaryC` encoding.
    pub fn add_binary(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u8]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Binary, out_type, field_tag)
            .raw_add_data_counted(field_value.as_ref());
    }

    /// Adds a Guid field from a `&Guid` value.
    ///
    /// GUID is assumed to be encoded in Windows (little-endian) byte order.
    ///
    /// If out_type is Default, field will format as Guid.
    pub fn add_guid(
        &mut self,
        field_name: &str,
        field_value: &Guid,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Guid, out_type, field_tag)
            .raw_add_data_value(field_value);
    }

    /// Adds a Guid variable-length array field from an iterator-of-`&Guid` value.
    ///
    /// GUID is assumed to be encoded in Windows (little-endian) byte order.
    ///
    /// If out_type is Default, field will format as Guid.
    pub fn add_guid_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a Guid>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::Guid, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a
    /// [FILETIME](https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime)
    /// field from an `i64` value.
    ///
    /// If out_type is Default, field will format as DateTime.
    /// Other useful out_type values: DateTimeCultureInsensitive, DateTimeUtc.
    ///
    /// You can use
    /// [`win_filetime_from_systemtime!`](crate::win_filetime_from_systemtime) macro to
    /// convert
    /// [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
    /// values into `i64` FILETIME values.
    pub fn add_filetime(
        &mut self,
        field_name: &str,
        field_value: i64,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::FileTime, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a
    /// [FILETIME](https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime)
    /// variable-length array field from an iterator-of-`&i64` value.
    ///
    /// If out_type is Default, field will format as DateTime.
    /// Other useful out_type values: DateTimeCultureInsensitive, DateTimeUtc.
    ///
    /// You can use
    /// [`win_filetime_from_systemtime!`](crate::win_filetime_from_systemtime) macro to
    /// convert
    /// [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
    /// values into `i64` FILETIME values.
    pub fn add_filetime_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a i64>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::FileTime, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a SystemTime field from a `&[u16; 8]` value.
    ///
    /// If out_type is Default, field will format as DateTime.
    /// Other useful out_type values: DateTimeCultureInsensitive, DateTimeUtc.
    pub fn add_systemtime(
        &mut self,
        field_name: &str,
        field_value: &[u16; 8],
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::SystemTime, out_type, field_tag)
            .raw_add_data_value(field_value);
    }

    /// Adds a SystemTime variable-length array field from an iterator-of-`&[u16; 8]` value.
    ///
    /// If out_type is Default, field will format as DateTime.
    /// Other useful out_type values: DateTimeCultureInsensitive, DateTimeUtc.
    pub fn add_systemtime_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a [u16; 8]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::SystemTime, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a Sid field from a `&[u8]` value.
    ///
    /// Sid size is determined by `8 + field_value[1] * 4`.
    ///
    /// If out_type is Default, field will format as SID.
    pub fn add_sid(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u8]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Sid, out_type, field_tag)
            .raw_add_data_sid(field_value.as_ref());
    }

    /// Adds a Sid variable-length array field from an iterator-of-`&[u8]` value.
    ///
    /// Sid size is determined by `8 + field_value[1] * 4`.
    ///
    /// If out_type is Default, field will format as SID.
    pub fn add_sid_sequence<T: IntoIterator>(
        &mut self,
        field_name: &str,
        field_values: T,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self
    where
        T::Item: AsRef<[u8]>,
    {
        return self
            .raw_add_meta_vcount(field_name, InType::Sid, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_sid(value.as_ref());
            });
    }

    /// Adds a Hex32 field from a `u32` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: Win32Error, NtStatus, CodePointer.
    pub fn add_hex32(
        &mut self,
        field_name: &str,
        field_value: u32,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Hex32, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a Hex32 variable-length array field from an iterator-of-`&u32` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: Win32Error, NtStatus, CodePointer.
    pub fn add_hex32_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a u32>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::Hex32, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a Hex64 field from a `u64` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: CodePointer.
    pub fn add_hex64(
        &mut self,
        field_name: &str,
        field_value: u64,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Hex64, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a Hex64 variable-length array field from an iterator-of-`&u64` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: CodePointer.
    pub fn add_hex64_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a u64>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::Hex64, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a HexSize field from a `usize` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: CodePointer.
    pub fn add_hexsize(
        &mut self,
        field_name: &str,
        field_value: usize,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::HexSize, out_type, field_tag)
            .raw_add_data_value(&field_value);
    }

    /// Adds a HexSize variable-length array field from an iterator-of-`&usize` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: CodePointer.
    pub fn add_hexsize_sequence<'a>(
        &mut self,
        field_name: &str,
        field_values: impl IntoIterator<Item = &'a usize>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_vcount(field_name, InType::HexSize, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_value(value);
            });
    }

    /// Adds a Str16 field (counted UTF16-LE) from a `&[u16]` value.
    ///
    /// If out_type is Default, field will format as String.
    /// Other useful out_type values: Xml, Json.
    ///
    /// This is the same as `add_cstr16` except that the ETW field will be encoded
    /// as a counted string instead of as a nul-terminated string. In most cases
    /// you should prefer this method and use `add_cstr16` only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_str16(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u16]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Str16, out_type, field_tag)
            .raw_add_data_counted(field_value.as_ref());
    }

    /// Adds a Str16 variable-length array field (counted UTF16-LE) from an iterator-of-`&[u16]` value.
    ///
    /// If out_type is Default, field will format as String.
    /// Other useful out_type values: Xml, Json.
    ///
    /// This is the same as `add_cstr16_sequence` except that the ETW field will be encoded
    /// as a counted string instead of as a nul-terminated string. In most cases
    /// you should prefer this method and use `add_cstr16_sequence` only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_str16_sequence<T: IntoIterator>(
        &mut self,
        field_name: &str,
        field_values: T,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self
    where
        T::Item: AsRef<[u16]>,
    {
        return self
            .raw_add_meta_vcount(field_name, InType::Str16, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_counted(value.as_ref());
            });
    }

    /// Adds a Str8 field (counted 8-bit string) from a `&[u8]` value.
    ///
    /// If out_type is Default, field will format as String (CP1252, not UTF-8).
    /// Other useful out_type values: Xml, Json, Utf8 (all of which decode as UTF-8).
    ///
    /// This is the same as `add_cstr8` except that the ETW field will be encoded
    /// as a counted string instead of as a nul-terminated string. In most cases
    /// you should prefer this method and use `add_cstr8` only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_str8(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u8]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::Str8, out_type, field_tag)
            .raw_add_data_counted(field_value.as_ref());
    }

    /// Adds a Str8 variable-length array field (counted 8-bit string) from an iterator-of-`&[u8]` value.
    ///
    /// If out_type is Default, field will format as String (CP1252, not UTF-8).
    /// Other useful out_type values: Xml, Json, Utf8 (all of which decode as UTF-8).
    ///
    /// This is the same as `add_cstr8_sequence` except that the ETW field will be encoded
    /// as a counted string instead of as a nul-terminated string. In most cases
    /// you should prefer this method and use `add_cstr8_sequence` only if you specifically
    /// need the nul-terminated encoding.
    pub fn add_str8_sequence<T: IntoIterator>(
        &mut self,
        field_name: &str,
        field_values: T,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self
    where
        T::Item: AsRef<[u8]>,
    {
        return self
            .raw_add_meta_vcount(field_name, InType::Str8, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_counted(value.as_ref());
            });
    }

    /// Adds a BinaryC field from a `&[u8]` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: IPv6, SocketAddress, Pkcs7WithTypeInfo.
    ///
    /// This is the same as add_binary, except that it uses a newer ETW encoding.
    /// - Newer ETW [InType], so decoding might not work with older decoders.
    /// - Decodes without the synthesized "FieldName.Length" field.
    /// - Arrays are supported.
    pub fn add_binaryc(
        &mut self,
        field_name: &str,
        field_value: impl AsRef<[u8]>,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        return self
            .raw_add_meta_scalar(field_name, InType::BinaryC, out_type, field_tag)
            .raw_add_data_counted(field_value.as_ref());
    }

    /// Adds a BinaryC variable-length array field from an iterator-of-`&[u8]` value.
    ///
    /// If out_type is Default, field will format as Hex.
    /// Other useful out_type values: IPv6, SocketAddress, Pkcs7WithTypeInfo.
    ///
    /// This is the same as add_binary, except that it uses a newer ETW encoding.
    /// - Newer ETW [InType], so decoding might not work with older decoders.
    /// - Decodes without the synthesized "FieldName.Length" field.
    /// - Arrays are supported.
    pub fn add_binaryc_sequence<T: IntoIterator>(
        &mut self,
        field_name: &str,
        field_values: T,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self
    where
        T::Item: AsRef<[u8]>,
    {
        return self
            .raw_add_meta_vcount(field_name, InType::BinaryC, out_type, field_tag)
            .raw_add_data_range(field_values, |this, value| {
                this.raw_add_data_counted(value.as_ref());
            });
    }

    /// Adds a Struct field with the specified number of nested fields.
    ///
    /// A struct is a way to logically group a number of fields. To add a struct to
    /// an event, call `builder.add_struct("StructName", field_count)`. Then add
    /// `field_count` more fields and they will be considered to be members of the
    /// struct. The `field_count` parameter must be in the range 0 to 127.
    ///
    /// Structs can nest. Each nested struct and its fields count as 1 field for the
    /// parent struct.
    pub fn add_struct(
        &mut self,
        field_name: &str,
        struct_field_count: u8,
        field_tag: u32,
    ) -> &mut Self {
        debug_assert_eq!(
            struct_field_count & OutType::TypeMask,
            struct_field_count,
            "struct_field_count must be less than 128"
        );
        return self.raw_add_meta(
            field_name,
            InType::Struct.as_int(),
            struct_field_count & OutType::TypeMask,
            field_tag,
        );
    }

    /// *Advanced scenarios:* Directly adds unchecked metadata to the event. Using this
    /// method may result in events that do not decode correctly.
    ///
    /// There are a few things that are supported by TraceLogging that cannot be expressed
    /// by directly calling the add methods, e.g. array-of-struct. If these edge cases are
    /// important, you can use the raw_add_meta and raw_add_data methods to generate events
    /// that would otherwise be impossible. Doing this requires advanced understanding of
    /// the TraceLogging encoding system. If done incorrectly, the resulting events will not
    /// decode properly.
    pub fn raw_add_meta_scalar(
        &mut self,
        field_name: &str,
        in_type: InType,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        debug_assert_eq!(
            in_type.as_int() & InType::FlagMask,
            0,
            "in_type must not include any flags"
        );
        return self.raw_add_meta(field_name, in_type.as_int(), out_type.as_int(), field_tag);
    }

    /// *Advanced scenarios:* Directly adds unchecked metadata to the event. Using this
    /// method may result in events that do not decode correctly.
    ///
    /// There are a few things that are supported by TraceLogging that cannot be expressed
    /// by directly calling the add methods, e.g. array-of-struct. If these edge cases are
    /// important, you can use the raw_add_meta and raw_add_data methods to generate events
    /// that would otherwise be impossible. Doing this requires advanced understanding of
    /// the TraceLogging encoding system. If done incorrectly, the resulting events will not
    /// decode properly.
    pub fn raw_add_meta_vcount(
        &mut self,
        field_name: &str,
        in_type: InType,
        out_type: OutType,
        field_tag: u32,
    ) -> &mut Self {
        debug_assert_eq!(
            in_type.as_int() & InType::FlagMask,
            0,
            "in_type must not include any flags"
        );
        return self.raw_add_meta(
            field_name,
            in_type.as_int() | InType::VariableCountFlag,
            out_type.as_int(),
            field_tag,
        );
    }

    /// *Advanced scenarios:* Directly adds unchecked data to the event. Using this
    /// method may result in events that do not decode correctly.
    ///
    /// There are a few things that are supported by TraceLogging that cannot be expressed
    /// by directly calling the add methods, e.g. array-of-struct. If these edge cases are
    /// important, you can use the raw_add_meta and raw_add_data methods to generate events
    /// that would otherwise be impossible. Doing this requires advanced understanding of
    /// the TraceLogging encoding system. If done incorrectly, the resulting events will not
    /// decode properly.
    pub fn raw_add_data_value<T: Copy>(&mut self, value: &T) -> &mut Self {
        let value_size = size_of::<T>();
        let old_data_size = self.data.len();
        self.data.reserve(value_size);
        unsafe {
            copy_nonoverlapping(
                value as *const T as *const u8,
                self.data.as_mut_ptr().add(old_data_size),
                value_size,
            );
            self.data.set_len(old_data_size + value_size);
        }
        return self;
    }

    /// *Advanced scenarios:* Directly adds unchecked data to the event. Using this
    /// method may result in events that do not decode correctly.
    ///
    /// There are a few things that are supported by TraceLogging that cannot be expressed
    /// by directly calling the add methods, e.g. array-of-struct. If these edge cases are
    /// important, you can use the raw_add_meta and raw_add_data methods to generate events
    /// that would otherwise be impossible. Doing this requires advanced understanding of
    /// the TraceLogging encoding system. If done incorrectly, the resulting events will not
    /// decode properly.
    pub fn raw_add_data_slice<T: Copy>(&mut self, value: &[T]) -> &mut Self {
        let value_size = value.len() * size_of::<T>();
        let old_data_size = self.data.len();
        self.data.reserve(value_size);
        unsafe {
            copy_nonoverlapping(
                value.as_ptr() as *const u8,
                self.data.as_mut_ptr().add(old_data_size),
                value_size,
            );
            self.data.set_len(old_data_size + value_size);
        }
        return self;
    }

    fn raw_add_meta(
        &mut self,
        field_name: &str,
        in_type: u8,
        out_type: u8,
        field_tag: u32,
    ) -> &mut Self {
        debug_assert!(
            !field_name.contains('\0'),
            "field_name must not contain '\\0'"
        );
        debug_assert_eq!(
            field_tag & 0x0FFFFFFF,
            field_tag,
            "field_tag must fit into 28 bits"
        );

        self.meta.reserve(field_name.len() + 7);

        self.meta.extend_from_slice(field_name.as_bytes());
        self.meta.push(0); // nul termination

        if field_tag != 0 {
            self.meta.push(0x80 | in_type);
            self.meta.push(0x80 | out_type);
            self.meta.push(0x80 | (field_tag >> 21) as u8);
            self.meta.push(0x80 | (field_tag >> 14) as u8);
            self.meta.push(0x80 | (field_tag >> 7) as u8);
            self.meta.push((0x7F & field_tag) as u8);
        } else if out_type != 0 {
            self.meta.push(0x80 | in_type);
            self.meta.push(out_type);
        } else {
            self.meta.push(in_type);
        }

        return self;
    }

    fn raw_add_data_sid(&mut self, value: &[u8]) -> &mut Self {
        let sid_length = 8 + 4 * (value[1] as usize);
        debug_assert!(
            sid_length <= value.len(),
            "add_sid(value) requires value.len() >= sid_length(value)"
        );
        return self.raw_add_data_slice(&value[0..sid_length]);
    }

    fn raw_add_data_cstr<T: Copy + Default + Eq>(&mut self, value: &[T]) -> &mut Self {
        let zero = T::default();
        let mut nul_pos = 0;
        while nul_pos != value.len() {
            if value[nul_pos] == zero {
                return self.raw_add_data_slice(&value[0..nul_pos + 1]);
            }
            nul_pos += 1;
        }

        return self.raw_add_data_slice(value).raw_add_data_value(&zero);
    }

    fn raw_add_data_counted<T: Copy>(&mut self, value: &[T]) -> &mut Self {
        let max_len = 65535 / size_of::<T>();
        if value.len() > max_len {
            return self
                .raw_add_data_value(&((max_len as u16) * (size_of::<T>() as u16)))
                .raw_add_data_slice(&value[0..max_len]);
        } else {
            return self
                .raw_add_data_value(&((value.len() as u16) * (size_of::<T>() as u16)))
                .raw_add_data_slice(value);
        }
    }

    fn raw_add_data_range<T: IntoIterator>(
        &mut self,
        field_values: T,
        add_data: impl Fn(&mut Self, T::Item),
    ) -> &mut Self {
        let mut count = 0u16;

        // Reserve space for count.
        let old_data_size = self.data.len();
        self.raw_add_data_value(&count);

        for value in field_values {
            if count == u16::MAX {
                break;
            }
            count += 1;
            add_data(self, value);
        }

        // Save actual value of count.
        self.data[old_data_size] = count as u8;
        self.data[old_data_size + 1] = (count >> 8) as u8;
        return self;
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        return Self::new();
    }
}
