//! Release history

#[allow(unused_imports)]
use crate::*; // For docs

/// # v2.0.0 (Unreleased, Breaking)
/// - Update `tracelogging` dependency to 2.0.0, which fixes data races (undefined
///   behavior) between the ETW enable callback and `Provider::enabled`, `register`, and
///   `unregister`, and makes an ETW enable level of 0 enable events of all levels.
/// - **Breaking:** `Provider::enabled` is no longer a `const fn`.
/// - New `kernel_mode` feature (enables `tracelogging/kernel_mode`). Required when
///   `tracelogging` is built with `kernel_mode` (enforced at compile time).
/// - **Breaking (kernel_mode only):** With the `kernel_mode` feature,
///   `Provider::unregister` is an `unsafe fn`. The caller must ensure that no other
///   thread is writing events with the provider while it is being unregistered.
///   Dropping the provider remains safe. In user mode it remains a safe fn.
/// - Provider, event, and field names containing `'\0'` now panic in release builds
///   too (previously only checked in debug builds, producing corrupt metadata in
///   release builds).
/// - Document that `Provider::unregister` does nothing if it overlaps with
///   `register`/`unregister` on other threads.
/// - `Provider::register` now sets the provider traits while holding the registration
///   guard, so an overlapping `unregister` cannot free the handle while the traits are
///   being set.
pub mod v2_0_0 {}

/// # v1.2.4 (2025-05-16)
/// - Fix linker issues when building with rustc 1.87
pub mod v1_2_4 {}

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
