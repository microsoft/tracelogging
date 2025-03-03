//! Release history

#[allow(unused_imports)]
use crate::*; // For docs

/// # v1.2.3 (2025-03-02)
/// - Fix newer warnings about unsafe code
/// - Update `tracelongging` dependency to 1.2.3
pub mod v1_2_3 {}

/// # v1.2.2 (2024-05-20)
/// - tracelogging crate supports use in kernel mode via feature
///   `kernel_mode`.
pub mod v1_2_2 {}

/// # v1.2.1 (2023-06-29)
/// - Guid: Fix `as_bytes_raw()` method and `borrow<[u8; 16]>` trait.
pub mod v1_2_1 {}

/// # v1.2.0 (2023-05-15)
/// - Guid: Added `as_bytes_raw()` method and `borrow<[u8; 16]>` trait.
/// - Opcode enum names clarified.
pub mod v1_2_0 {}

/// # v1.1.0 (2023-03-24, Breaking)
/// - **Breaking:** Move [`Provider`] configuration parameters from
///   [`Provider::register`] to [`Provider::new`]. This allows
///   [`Provider::register`] to be an immutable operation, making the
///   `Provider` type easier to use in multi-threaded scenarios.
/// - Relax multithreading precondition on [`Provider::unregister`]. Now,
///   only [`Provider::register`] has special preconditions.
pub mod v1_1_0 {}

/// # v1.0.2 (2023-03-13, Breaking)
/// - **Breaking:** Rename the `filetime_from_systemtime` macro to
///   [`win_filetime_from_systemtime`].
/// - Improve the doc comments for [`win_filetime_from_systemtime`].
/// - Define `Debug` trait on [`EventBuilder`].
/// - Define `Debug` trait on [`ProviderOptions`].
pub mod v1_0_2 {}

/// # v1.0.1 (2023-03-13)
/// - Add `filetime_from_systemtime` macro.
pub mod v1_0_1 {}

/// # v0.1.0 (2022-08-13)
/// - Initial release.
pub mod v0_1_0 {}
