// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![doc(hidden)]
//! Internal implementation details for tracelogging macros and tracelogging_dynamic.
//! Contents subject to change without notice.

use core::mem;
use core::slice;
use core::time::Duration;

pub use crate::descriptors::counted_size;
pub use crate::descriptors::slice_count;
pub use crate::descriptors::EventDataDescriptor;
pub use crate::descriptors::EventDescriptor;
pub use crate::native::ProviderContext;
pub use crate::provider::provider_new;
pub use crate::provider::provider_write_transfer;

/// Returns the metadata bytes for the given metadata structure.
pub fn meta_as_bytes<T>(meta: &T) -> &[u8] {
    // Safety: read-only, pointer and size are valid.
    unsafe { slice::from_raw_parts(meta as *const T as *const u8, mem::size_of::<T>()) }
}

/// Returns the number of bytes needed to encode the specified tag.
pub const fn tag_size(tag: u32) -> usize {
    if 0 == (tag & 0x001FFFFF) {
        1
    } else if 0 == (tag & 0x00003FFF) {
        2
    } else if 0 == (tag & 0x0000007F) {
        3
    } else {
        4
    }
}

/// Returns the encoded tag.
pub const fn tag_encode<const SIZE: usize>(tag: u32) -> [u8; SIZE] {
    assert!(SIZE != 0);
    let mut result = [0; SIZE];

    let mut bits = tag;
    let mut i = 0;
    while i != SIZE {
        result[i] = ((bits >> 21) & 0x7F) as u8;
        bits <<= 7;

        if i != SIZE - 1 {
            result[i] |= 0x80;
        }

        i += 1;
    }

    result
}

/// Returns the filetime corresponding to a duration returned by
/// `systemtime.duration_since(SystemTime::UNIX_EPOCH)`.
/// The ok parameter should be true if duration_since returned Ok, false if Err.
/// ```
/// # use tracelogging::_internal as tli;
/// # use std::time::SystemTime;
/// let systemtime = SystemTime::now();
/// let filetime = match systemtime.duration_since(SystemTime::UNIX_EPOCH) {
///     Ok(dur) => tli::filetime_from_duration_since_1970(dur, true),
///     Err(err) => tli::filetime_from_duration_since_1970(err.duration(), false),
/// };
/// ```
pub const fn filetime_from_duration_since_1970(duration: Duration, ok: bool) -> i64 {
    const FILETIME_MIN: u64 = 0; // FileTimeToSystemTime does not support negative FILETIMEs.
    const FILETIME_MAX: u64 = 0x7FFF35F4F06C7FFF; // Highest value supported by FileTimetoSystemTime
    const UNIX_EPOCH_FILETIME: u64 = 0x19DB1DED53E8000;
    const FILETIME_PER_SECOND: u64 = 10000000;
    const SECS_MAX_OK: u64 = (FILETIME_MAX - UNIX_EPOCH_FILETIME) / FILETIME_PER_SECOND;
    const SECS_MAX_ERR: u64 = (UNIX_EPOCH_FILETIME - FILETIME_MIN) / FILETIME_PER_SECOND - 1;
    let secs = duration.as_secs();
    let ticks = secs
        .wrapping_mul(FILETIME_PER_SECOND)
        .wrapping_add((duration.subsec_nanos() / 100) as u64);
    return if ok {
        if secs <= SECS_MAX_OK {
            UNIX_EPOCH_FILETIME.wrapping_add(ticks)
        } else {
            FILETIME_MAX
        }
    } else if secs <= SECS_MAX_ERR {
        UNIX_EPOCH_FILETIME.wrapping_add(ticks.wrapping_neg())
    } else {
        FILETIME_MIN
    } as i64;
}
