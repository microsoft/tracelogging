// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use core::fmt;
use core::str::from_utf8;

use crate::descriptors::EventDataDescriptor;
use crate::descriptors::EventDescriptor;
use crate::enums::Level;
use crate::guid::Guid;
use crate::native::ProviderContext;
use crate::native::ProviderEnableCallback;

#[allow(unused_imports)] // For docs
#[cfg(feature = "macros")]
use crate::define_provider;

#[allow(unused_imports)] // For docs
#[cfg(feature = "macros")]
use crate::write_event;

/// A connection to ETW for writing TraceLogging (manifest-free) events.
///
/// # Overview
///
/// 1. Use [`define_provider!`] to create a static provider variable.
/// 2. Call [`Provider::register()`] during component initialization to open the
///    connection to ETW.
/// 3. Use [`write_event!`] as needed to write events.
/// 4. Call [`Provider::unregister()`] during component cleanup to close the connection
///    to ETW.
pub struct Provider {
    context: ProviderContext,
    meta: &'static [u8], // provider metadata
    id: Guid,
}

impl Provider {
    /// Returns the current thread's thread-local activity id.
    /// (Calls
    /// [EventActivityIdControl](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventactivityidcontrol)
    /// with control code `EVENT_ACTIVITY_CTRL_GET_ID`.)
    ///
    /// The thread-local activity id will be used if [`write_event!`] is called with no
    /// `activity_id` option.
    pub fn current_thread_activity_id() -> Guid {
        let mut activity_id = Guid::default();
        ProviderContext::activity_id_control(
            1, // GetId
            &mut activity_id,
        );
        return activity_id;
    }

    /// Sets the current thread's thread-local activity id. Returns the previous value.
    /// (Calls
    /// [EventActivityIdControl](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventactivityidcontrol)
    /// with control code `EVENT_ACTIVITY_CTRL_GET_SET_ID`.)
    ///
    /// The thread-local activity id will be used if [`write_event!`] is called with no
    /// `activity_id` option.
    ///
    /// Important: thread-local activity id should follow scoping rules. If you set the
    /// thread-local activity id in a scope, you should restore the previous value before exiting
    /// the scope.
    pub fn set_current_thread_activity_id(value: &Guid) -> Guid {
        let mut activity_id = *value;
        ProviderContext::activity_id_control(
            4, // GetSetId
            &mut activity_id,
        );
        return activity_id;
    }

    /// Generates and returns a new 128-bit value suitable for use as an activity id.
    /// (Calls
    /// [EventActivityIdControl](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventactivityidcontrol)
    /// with control code `EVENT_ACTIVITY_CREATE_ID`.)
    ///
    /// The returned value is not a true GUID/UUID since it is not globally-unique. The
    /// returned value is locally-unique: it is guaranteed to be different from all other
    /// values generated by create_activity_id() on the current machine in the current
    /// boot session.
    ///
    /// Activity ids need to be unique within the trace but do not need to be
    /// globally-unique, so your activity ids can use either a real GUID/UUID or a
    /// locally-unique id generated by create_activity_id(). Use create_activity_id() for
    /// locally-unique activity ids or use [Guid::new] for globally-unique activity ids.
    pub fn create_activity_id() -> Guid {
        let mut activity_id = Guid::default();
        ProviderContext::activity_id_control(
            3, // CreateId
            &mut activity_id,
        );
        return activity_id;
    }

    /// Returns a GUID generated from a case-insensitive hash of the specified trace
    /// provider name. The hash uses the same algorithm as many other ETW tools and APIs.
    /// Given the same name, it will always generate the same GUID.
    /// (Same as [Guid::from_name].)
    ///
    /// The result can (and usually should) be used as the provider id.
    ///
    /// **Note:** Do not use guid_from_name to generate activity ids.
    /// ```
    /// use tracelogging as tlg;
    /// assert_eq!(
    ///    tlg::Provider::guid_from_name("MyProvider"),
    ///    tlg::Guid::from_u128(&0xb3864c38_4273_58c5_545b_8b3608343471));
    /// ```
    pub fn guid_from_name(name: &str) -> Guid {
        return Guid::from_name(name);
    }

    /// *Advanced:* Returns this provider's encoded metadata bytes.
    pub const fn raw_meta(&self) -> &[u8] {
        return self.meta;
    }

    /// Returns this provider's name.
    pub fn name(&self) -> &str {
        let mut name_end = 2;
        while self.meta[name_end] != 0 {
            name_end += 1;
        }
        return from_utf8(&self.meta[2..name_end]).unwrap();
    }

    /// Returns this provider's id (GUID).
    pub const fn id(&self) -> &Guid {
        return &self.id;
    }

    /// Returns true if any ETW logging session is listening to this provider for events
    /// with the specified level and keyword.
    ///
    /// Note: [`write_event!`] already checks `enabled()`. You only need to make your own
    /// call to `enabled()` if you want to skip something other than [`write_event!`].
    #[inline(always)]
    pub const fn enabled(&self, level: Level, keyword: u64) -> bool {
        return self.context.enabled(level, keyword);
    }

    /// If this provider is not registered, does nothing and returns 0.
    /// Otherwise, unregisters the provider.
    ///
    /// Returns 0 for success or a Win32 error from `EventUnregister` for failure. The
    /// return value is for diagnostic purposes only and should generally be ignored in
    /// retail builds.
    pub fn unregister(&self) -> u32 {
        return self.context.unregister();
    }

    /// Register the provider.
    ///
    /// # Preconditions
    ///
    /// - Provider must not already be registered. Verified at runtime, failure = panic.
    /// - For a given provider object, a call on one thread to the provider's `register`
    ///   method must not occur at the same time as a call to the same provider's
    ///   `register` or `unregister` method on any other thread. Verified at runtime,
    ///   failure = panic.
    ///
    /// # Safety
    ///
    /// - If creating a DLL or creating a provider that might run as part of a DLL, all
    ///   registered providers **must** be unregistered before the DLL unloads.
    ///
    ///   If a provider variable is registered by a DLL and the DLL unloads while the
    ///   provider is still registered, the process may subsequently crash. This occurs
    ///   because `register` enables an ETW callback into the calling DLL and
    ///   `unregister` ensures that the callback is disabled. If the module unloads
    ///   without disabling the callback, the process will crash the next time that ETW
    ///   tries to invoke the callback.
    ///
    ///   The provider cannot unregister itself because the provider is static and Rust
    ///   does not drop static objects.
    ///
    ///   You'll typically register the provider during `DLL_PROCESS_ATTACH` and
    ///   unregister during `DLL_PROCESS_DETACH`.
    pub unsafe fn register(&self) -> u32 {
        return self.register_impl(None, 0);
    }

    /// Register the provider with a custom provider enable callback.
    ///
    /// # Preconditions
    ///
    /// - Provider must not already be registered. Verified at runtime, failure = panic.
    /// - For a given provider object, a call on one thread to the provider's `register`
    ///   method must not occur at the same time as a call to the same provider's
    ///   `register` or `unregister` method on any other thread. Verified at runtime,
    ///   failure = panic.
    ///
    /// # Safety
    ///
    /// - If creating a DLL or creating a provider that might run as part of a DLL, all
    ///   registered providers **must** be unregistered before the DLL unloads.
    ///
    ///   If a provider variable is registered by a DLL and the DLL unloads while the
    ///   provider is still registered, the process may subsequently crash. This occurs
    ///   because `register` enables an ETW callback into the calling DLL and
    ///   `unregister` ensures that the callback is disabled. If the module unloads
    ///   without disabling the callback, the process will crash the next time that ETW
    ///   tries to invoke the callback.
    ///
    ///   The provider cannot unregister itself because the provider is static and Rust
    ///   does not drop static objects.
    ///
    ///   You'll typically register the provider during `DLL_PROCESS_ATTACH` and
    ///   unregister during `DLL_PROCESS_DETACH`.
    pub unsafe fn register_with_callback(
        &self,
        callback_fn: ProviderEnableCallback,
        callback_context: usize,
    ) -> u32 {
        return self.register_impl(Some(callback_fn), callback_context);
    }

    /// Safety:
    ///
    /// 1. Pinning: The only way to construct a provider is `provider_new`.
    ///    `provider_new` is unsafe and declares its safety condition as the provider
    ///    must not be moved-from while registered, so we're covered for any direct
    ///    use of `provider_new`. The `new_provider!` macro calls `provider_new` and
    ///    prohibits moving by storing the provider in an immutable variable, so
    ///    `new_provider!` is meeting its obligations.
    ///
    /// 2. Unregister: Our `register` and `register_with_callback` methods are unsafe and
    ///    declare safety conditions matching the Unregister condition.
    fn register_impl(
        &self,
        callback_fn: Option<ProviderEnableCallback>,
        callback_context: usize,
    ) -> u32 {
        let result = unsafe {
            self.context
                .register(&self.id, callback_fn, callback_context)
        };

        if result == 0 {
            // 2 == EventProviderSetTraits
            self.context.set_information(2, self.meta);
        }

        return result;
    }
}

impl fmt::Debug for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "Provider {{ name: \"{}\", id: {}, enabled: {}, reg_handle: {:x} }}",
            self.name(),
            from_utf8(&self.id.to_utf8_bytes()).unwrap(),
            self.enabled(Level::LogAlways, 0),
            self.context.reg_handle()
        );
    }
}

/// For use by the define_provider macro: creates a new provider.
///
/// # Safety
///
/// - Must not move-out of a provider while it is registered. `define_provider` enforces
///   this by storing the result in an immutable variable.
pub const unsafe fn provider_new(meta: &'static [u8], id: &Guid) -> Provider {
    return Provider {
        context: ProviderContext::new(),
        meta,
        id: *id,
    };
}

/// For use by the write_event macro: Calls EventWriteTransfer.
pub fn provider_write_transfer(
    provider: &Provider,
    descriptor: &EventDescriptor,
    activity_id: Option<&Guid>,
    related_id: Option<&Guid>,
    dd: &[EventDataDescriptor],
) -> u32 {
    return provider
        .context
        .write_transfer(descriptor, activity_id, related_id, dd);
}
