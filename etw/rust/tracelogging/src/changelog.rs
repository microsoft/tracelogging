//! Release history

#[allow(unused_imports)]
use crate::*; // For docs

/// # v1.0.3 (2023-03-23)
/// - Relax multithreading precondition on [`Provider::unregister`]. Now,
///   only [`Provider::register`] has special preconditions.
pub mod v1_0_3 {}

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
