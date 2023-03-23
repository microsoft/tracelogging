//! Release history

#[allow(unused_imports)]
use crate::*; // For docs

/// # v1.0.3 (2023-03-23, Breaking)
/// - **Breaking:** Move [`Provider`] configuration parameters from
///   [`Provider::register`] to [`Provider::new`]. This allows
///   [`Provider::register`] to be an immutable operation, making the
///   `Provider` type easier to use in multi-threaded scenarios.
/// - Relax multithreading precondition on [`Provider::unregister`]. Now,
///   only [`Provider::register`] has special preconditions.
pub mod v1_0_3 {}

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
