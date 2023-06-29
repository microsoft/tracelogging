//! Release history

#[allow(unused_imports)]
use crate::*; // For docs

/// # v1.2.1 (2023-06-29)
/// - Guid: Fix `as_bytes_raw()` method and `borrow<[u8; 16]>` trait.
pub mod v1_2_1 {}

/// # v1.2.0 (2023-05-15)
/// - In event macros, `activity_id` and `related_id` values can now be either
///   `&Guid` or `&[u8; 16]`.
/// - New event macro field types `errno`, `errno_slice`, `time32`, and `time64`
///   (for compatibility with eventheader).
/// - New provider macro option `group_name` (for compatibility with eventheader).
/// - Guid: Added `as_bytes_raw()` method and `borrow<[u8; 16]>` trait.
/// - Better macro parse error reporting.
/// - Opcode enum names clarified.
pub mod v1_2_0 {}

/// # v1.1.0 (2023-03-24)
/// - Relax multithreading precondition on [`Provider::unregister`]. Now,
///   only [`Provider::register`] has special preconditions.
pub mod v1_1_0 {}

/// # v1.0.2 (2023-03-13, Breaking)
/// - **Breaking:** Rename the `filetime_from_systemtime` macro to
///   [`win_filetime_from_systemtime`].
/// - Improve the doc comments for [`win_filetime_from_systemtime`].
/// - Remove "For testing" macros from `tracelogging_macros` crate.
pub mod v1_0_2 {}

/// # v1.0.1 (2023-03-13)
/// - Add `filetime_from_systemtime` macro.
/// - Optimization: Split internal `filetime_from_duration` function into separate
///   before-1970 and after-1970 functions.
pub mod v1_0_1 {}

/// # v0.1.0 (2022-08-13)
/// - Initial release.
pub mod v0_1_0 {}
