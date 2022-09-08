// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use core::marker::PhantomData;
use core::mem::size_of;

use crate::enums::Channel;
use crate::enums::Level;
use crate::enums::Opcode;

/// Characteristics of an ETW event: severity level, category bits, etc.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EventDescriptor {
    /// Set id to 0 unless the event has a manually-assigned stable id.
    pub id: u16,

    /// Set version to 0 unless the event has a manually-assigned stable id.
    /// If the event does have a manually-assigned stable id, start the version
    /// at 0, then increment the version for each breaking change to the event
    /// (e.g. changes to the field names, types, or semantics).
    pub version: u8,

    /// For TraceLogging events, channel should usually be set to
    /// [Channel::TraceLogging] (11). TraceLogging events with channel set to a value
    /// other than 11 might not decode correctly if they are collected on a system
    /// running Windows 8.1 or before.
    ///
    /// For non-TraceLogging events, channel should usually be set to
    /// [Channel::TraceClassic] (0).
    pub channel: Channel,

    /// Event severity level: 1=critical, 2=error, 3=warning, 4=info, 5=verbose.
    /// If unsure, use 5 (verbose).
    pub level: Level,

    /// Special semantics for event: 0=informational, 1=activity-start, 2=activity-stop.
    pub opcode: Opcode,

    /// Provider-defined event task. Usually set to 0.
    pub task: u16,

    /// keyword is a bit mask of category bits. The upper 16 bits are defined by
    /// Microsoft. The low 48 bits are defined by the user on a per-provider basis, i.e.
    /// all providers with a particular name and id should use the same category bit
    /// assignments. For example, your provider might define 0x2 as the "networking"
    /// category bit and 0x4 as the "threading" category bit.
    ///
    /// Keyword should always be a non-zero value. If category bits have not been defined
    /// for your provider, define 0x1 as "uncategorized" and use 0x1 as the keyword for
    /// all events.
    pub keyword: u64,
}

impl EventDescriptor {
    /// Creates a new descriptor with all zero values (channel = TraceClassic).
    /// The resulting descriptor should not usually be used until the values
    /// for level, keyword, and channel have been updated.
    pub const fn zero() -> EventDescriptor {
        return EventDescriptor {
            id: 0,
            version: 0,
            channel: Channel::TraceClassic,
            level: Level::LogAlways,
            opcode: Opcode::Info,
            task: 0,
            keyword: 0,
        };
    }

    /// Creates a new descriptor for an informational event (channel = TraceLogging).
    ///
    /// level: critical, error, warning, info, verbose; if unsure use verbose.
    ///
    /// keyword: event category bits; if unsure use 0x1.
    pub const fn new(level: Level, keyword: u64) -> EventDescriptor {
        return EventDescriptor {
            id: 0,
            version: 0,
            channel: Channel::TraceLogging,
            level,
            opcode: Opcode::Info,
            task: 0,
            keyword,
        };
    }

    /// Creates a new descriptor from values.
    pub const fn from_parts(
        id: u16,
        version: u8,
        channel: Channel,
        level: Level,
        opcode: Opcode,
        task: u16,
        keyword: u64,
    ) -> EventDescriptor {
        return EventDescriptor {
            id,
            version,
            channel,
            level,
            opcode,
            task,
            keyword,
        };
    }
}

/// Describes a block of data to be sent to ETW via EventWrite.
#[repr(C)]
#[derive(Debug, Default)]
pub struct EventDataDescriptor<'a> {
    ptr: u64,
    size: u32,
    reserved: u32,
    lifetime: PhantomData<&'a [u8]>,
}

impl<'a> EventDataDescriptor<'a> {
    /// Returns an EventDataDescriptor initialized with the specified slice's bytes and
    /// the specified value in the reserved field.
    pub fn from_raw_bytes<'v: 'a>(value: &'v [u8], reserved: u32) -> Self {
        return Self {
            ptr: value.as_ptr() as usize as u64,
            size: value.len() as u32,
            reserved,
            lifetime: PhantomData,
        };
    }

    /// Returns an EventDataDescriptor initialized with the specified value's bytes.
    /// Sets the reserved field to 0.
    pub fn from_value<'v: 'a, T: Copy>(value: &'v T) -> Self {
        return Self {
            ptr: value as *const T as usize as u64,
            size: size_of::<T>() as u32,
            reserved: 0,
            lifetime: PhantomData,
        };
    }

    /// Returns an EventDataDescriptor for a sid.
    /// Sets the reserved field to 0.
    /// Requires value.len() >= sid_length(value).
    pub fn from_sid<'v: 'a>(mut value: &'v [u8]) -> Self {
        let value_len = value.len();

        debug_assert!(4 <= value_len, "add_sid(value) requires value.len() >= 4");
        let len = 8 + 4 * (value[1] as usize); // May panic if precondition not met.

        if len != value_len {
            debug_assert!(
                len <= value_len,
                "add_sid(value) requires value.len() >= sid_length(value)"
            );
            value = &value[..len]; // May panic if precondition not met.
        }

        return Self {
            ptr: value.as_ptr() as usize as u64,
            size: (size_of::<u8>() as u32) * (value.len() as u32),
            reserved: 0,
            lifetime: PhantomData,
        };
    }

    /// Returns an EventDataDescriptor for a nul-terminated string.
    /// Sets the reserved field to 0.
    /// Returned descriptor does not include the nul-termination.
    pub fn from_cstr<'v: 'a, T: Copy + Default + Eq>(mut value: &'v [T]) -> Self {
        let mut value_len = value.len();

        const MAX_LEN: usize = 65535;
        if value_len > MAX_LEN {
            value = &value[..MAX_LEN];
            value_len = value.len();
        }

        let zero = T::default();
        let mut len = 0;
        while len < value_len {
            if value[len] == zero {
                value = &value[..len];
                break;
            }

            len += 1;
        }

        return Self {
            ptr: value.as_ptr() as usize as u64,
            size: (size_of::<T>() as u32) * (value.len() as u32),
            reserved: 0,
            lifetime: PhantomData,
        };
    }

    /// Returns an EventDataDescriptor for a counted field (string or binary).
    /// Sets the reserved field to 0.
    pub fn from_counted<'v: 'a, T: Copy>(mut value: &'v [T]) -> Self {
        let value_len = value.len();

        let max_len = 65535 / size_of::<T>();
        if max_len < value_len {
            value = &value[0..max_len];
        }

        return Self {
            ptr: value.as_ptr() as usize as u64,
            size: (size_of::<T>() as u32) * (value.len() as u32),
            reserved: 0,
            lifetime: PhantomData,
        };
    }

    /// Returns an EventDataDescriptor for variable-length array field.
    /// Sets the reserved field to 0.
    pub fn from_slice<'v: 'a, T: Copy + 'v>(mut value: &'v [T]) -> Self {
        let value_len = value.len();

        const MAX_LEN: usize = 65535;
        if MAX_LEN < value_len {
            value = &value[..MAX_LEN];
        }

        return Self {
            ptr: value.as_ptr() as usize as u64,
            size: (size_of::<T>() as u32) * (value.len() as u32),
            reserved: 0,
            lifetime: PhantomData,
        };
    }
}

/// Returns the size for a counted field.
pub fn counted_size<T>(value: &[T]) -> u16 {
    let value_len = value.len();
    let max_len = (65535 / size_of::<T>()) as u16;
    let len = safe_len(max_len, value_len);
    return (size_of::<T>() as u16) * len;
}

/// Returns the count for a variable-length array field.
pub fn slice_count<T>(value: &[T]) -> u16 {
    return safe_len(65535, value.len());
}

const fn safe_len(max_len: u16, len: usize) -> u16 {
    return if (max_len as usize) < len {
        max_len
    } else {
        len as u16
    };
}
