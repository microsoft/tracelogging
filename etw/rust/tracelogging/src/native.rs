// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use core::marker::PhantomPinned;

#[cfg(all(windows, feature = "etw"))]
use core::sync::atomic;

use crate::descriptors::EventDataDescriptor;
use crate::descriptors::EventDescriptor;
use crate::enums::Level;
use crate::guid::Guid;

/// Possible configurations under which this crate can be compiled: `Windows` or `Other`.
pub enum NativeImplementation {
    /// Crate compiled for other configuration (no logging is performed).
    Other,
    /// Crate compiled for Windows (ETW) configuration (logging is performed via
    /// user-mode ETW APIs like EventWriteTransfer).
    Windows,
    /// Crate compiled for Windows (ETW) kernel-mode clients (logging is performed via
    /// kernel-mode ETW APIs like EtwWriteTransfer).
    ///
    WindowsKernelMode,
}

/// The configuration under which this crate was compiled: `Windows`, `WindowsKernelMode` or `Other`.
pub const NATIVE_IMPLEMENTATION: NativeImplementation = if cfg!(all(windows, feature = "etw", not(feature = "kernel_mode"))) {
    NativeImplementation::Windows
} else if cfg!(all(windows, feature = "etw", feature = "kernel_mode")) {
    NativeImplementation::WindowsKernelMode
}
else {
    NativeImplementation::Other
};

/// Signature for a custom
/// [provider enable callback](https://docs.microsoft.com/windows/win32/api/evntprov/nc-evntprov-penablecallback).
pub type ProviderEnableCallback = fn(
    source_id: &Guid,
    event_control_code: u32,
    level: Level,
    match_any_keyword: u64,
    match_all_keyword: u64,
    filter_data: usize,
    callback_context: usize,
);

#[cfg(all(windows, feature = "etw"))]
type OuterEnableCallback = unsafe extern "system" fn(
    source_id: &Guid,
    event_control_code: u32,
    level: u8,
    match_any_keyword: u64,
    match_all_keyword: u64,
    filter_data: usize,
    outer_context: usize, // = &ProviderContextInner
);

/// Data needed to manage an ETW registration with callback.
pub struct ProviderContext {
    _pinned: PhantomPinned,

    #[cfg(all(windows, feature = "etw"))]
    inner: ProviderContextInner,
}

impl ProviderContext {
    /// Windows: return EventActivityIdControl(...);
    /// WindowsKernelMode: return EtwActivityIdControl(...);
    /// Other: return ERROR_NOT_SUPPORTED;
    pub fn activity_id_control(_control_code: u32, _activity_id: &mut Guid) -> u32 {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = 50; // ERROR_NOT_SUPPORTED
        }
        #[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
        {
            result = unsafe { EventActivityIdControl(_control_code, _activity_id) };
        }
        #[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
        {
            result = unsafe { EtwActivityIdControl(_control_code, _activity_id) };
        }
        return result;
    }

    /// Creates a new provider context.
    pub const fn new() -> ProviderContext {
        return ProviderContext {
            _pinned: PhantomPinned,

            #[cfg(all(windows, feature = "etw"))]
            inner: ProviderContextInner::new(),
        };
    }

    /// Returns the registration handle. For diagnostic purposes only.
    pub fn reg_handle(&self) -> u64 {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = 0;
        }
        #[cfg(all(windows, feature = "etw"))]
        {
            result = self.inner.reg_handle.load(atomic::Ordering::Acquire);
        }
        return result;
    }

    /// Returns true if the provider is enabled at the specified level and keyword.
    #[inline(always)]
    pub fn enabled(&self, _level: Level, _keyword: u64) -> bool {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = false;
        }
        #[cfg(all(windows, feature = "etw"))]
        {
            result = self.inner.enabled(_level, _keyword);
        }
        return result;
    }

    /// Calls EventUnregister (EtwUnregister for kernel_mode) and sets reg_handle = 0.
    ///
    /// When the `kernel_mode` feature is enabled, this method is `unsafe`; see the
    /// `kernel_mode` version's Safety section.
    ///
    /// # Preconditions
    /// - If this call overlaps a call to `register` or `unregister` on another thread,
    ///   it does nothing and returns 0 immediately. In that case the provider may still
    ///   be registered (overlap with `register`) or the other thread's unregistration
    ///   may still be in progress (overlap with `unregister`) when this call returns.
    #[cfg(not(feature = "kernel_mode"))]
    pub fn unregister(&self) -> u32 {
        return self.unregister_impl();
    }

    /// Calls EtwUnregister and sets reg_handle = 0.
    ///
    /// # Preconditions
    /// - If this call overlaps a call to `register` or `unregister` on another thread,
    ///   it does nothing and returns 0 immediately. In that case the provider may still
    ///   be registered (overlap with `register`) or the other thread's unregistration
    ///   may still be in progress (overlap with `unregister`) when this call returns.
    ///
    /// # Safety
    /// - Must not be called while another thread may be calling `write_transfer` or
    ///   `set_information` on this context. A concurrent write may pass an
    ///   already-unregistered handle to EtwWriteTransfer, which is a use-after-free.
    #[cfg(feature = "kernel_mode")]
    pub unsafe fn unregister(&self) -> u32 {
        return self.unregister_impl();
    }

    fn unregister_impl(&self) -> u32 {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = 0;
        }
        #[cfg(all(windows, feature = "etw"))]
        {
            result = self.inner.unregister();
        }
        return result;
    }

    /// Calls EventRegister (EtwRegister for kernel_mode). If registration succeeds and
    /// `provider_traits` is not empty, also calls EventSetInformation
    /// (EtwSetInformation for kernel_mode) with `EventProviderSetTraits` before the
    /// registration handle is published, all while holding the registration guard, so
    /// the traits are set before any events are written and cannot race with
    /// `unregister`.
    ///
    /// # Preconditions
    /// - This will panic if provider is currently registered.
    /// - This will panic if it overlaps with another thread simultaneously calling
    ///   register or unregister.
    ///
    /// # Safety
    /// 1. Pinning: Context must not be moved-from as long as provider is registered.
    /// 2. Unregister: If a provider is registered in a DLL or driver, unregister
    ///    **must** be called before the module unloads.
    pub unsafe fn register(
        &self,
        _provider_id: &Guid,
        _callback_fn: Option<ProviderEnableCallback>,
        _callback_context: usize,
        _provider_traits: &[u8],
    ) -> u32 {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = 0;
        }
        #[cfg(all(windows, feature = "etw"))]
        {
            result = self.inner.register(
                _provider_id,
                _callback_fn,
                _callback_context,
                _provider_traits,
            );
        }
        return result;
    }

    /// Calls EventSetInformation (EtwSetInformation for kernel_mode).
    pub fn set_information(&self, _information_class: u32, _information: &[u8]) -> u32 {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = 0;
        }
        #[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
        {
            result = unsafe {
                EventSetInformation(
                    self.reg_handle(),
                    _information_class,
                    _information.as_ptr(),
                    _information.len() as u32,
                )
            };
        }
        #[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
        {
            result = unsafe {
                EtwSetInformation(
                    self.reg_handle(),
                    _information_class,
                    _information.as_ptr(),
                    _information.len() as u32,
                )
            };
        }
        return result;
    }

    /// Calls EventWriteTransfer (EtwWriteTransfer for kernel_mode).
    pub fn write_transfer(
        &self,
        _descriptor: &EventDescriptor,
        _activity_id: Option<&[u8; 16]>,
        _related_id: Option<&[u8; 16]>,
        _data: &[EventDataDescriptor],
    ) -> u32 {
        let result;
        #[cfg(not(all(windows, feature = "etw")))]
        {
            result = 0;
        }
        #[cfg(all(windows, feature = "etw"))]
        let reg_handle = self.reg_handle();
        #[cfg(all(windows, feature = "etw"))]
        if reg_handle == 0 {
            // Not registered, or enabled by a callback before register has published
            // the handle.
            return 0;
        }
        #[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
        {
            result = unsafe {
                EventWriteTransfer(
                    reg_handle,
                    _descriptor,
                    _activity_id,
                    _related_id,
                    _data.len() as u32,
                    _data.as_ptr(),
                )
            };
        }
        #[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
        {
            result = unsafe {
                EtwWriteTransfer(
                    reg_handle,
                    _descriptor,
                    _activity_id,
                    _related_id,
                    _data.len() as u32,
                    _data.as_ptr(),
                )
            };
        }
        return result;
    }
}

impl Default for ProviderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProviderContext {
    /// Calls unregister.
    fn drop(&mut self) {
        // `&mut self` guarantees no other thread is using this context, so no event
        // write can overlap the unregistration (the kernel_mode safety requirement).
        self.unregister_impl();
    }
}

#[cfg(all(windows, feature = "etw"))]
struct ProviderContextInner {
    level: atomic::AtomicI32, // -1 means not enabled by anybody.
    busy: atomic::AtomicBool,
    reg_handle: atomic::AtomicU64,
    keyword_any: atomic::AtomicU64,
    keyword_all: atomic::AtomicU64,
    callback_fn: atomic::AtomicPtr<()>, // null or a ProviderEnableCallback.
    callback_context: atomic::AtomicUsize,
}

#[cfg(all(windows, feature = "etw"))]
impl ProviderContextInner {
    const fn new() -> Self {
        return Self {
            level: atomic::AtomicI32::new(-1),
            busy: atomic::AtomicBool::new(false),
            reg_handle: atomic::AtomicU64::new(0),
            keyword_any: atomic::AtomicU64::new(0),
            keyword_all: atomic::AtomicU64::new(0),
            callback_fn: atomic::AtomicPtr::new(core::ptr::null_mut()),
            callback_context: atomic::AtomicUsize::new(0),
        };
    }

    /// Returns true if the provider is enabled at the specified level and keyword.
    #[inline(always)]
    fn enabled(&self, level: Level, keyword: u64) -> bool {
        // Load level first: this Acquire pairs with the Release store of level in
        // outer_callback_impl (which is done after the keyword stores), so the keywords
        // read below are at least as new as the level. The early return also means the
        // common "not enabled" case costs a single load.
        let enabled_level = self.level.load(atomic::Ordering::Acquire);
        if (level.0 as i32) > enabled_level {
            return false;
        }

        if keyword == 0 {
            return true;
        }

        let keyword_any = self.keyword_any.load(atomic::Ordering::Relaxed);
        let keyword_all = self.keyword_all.load(atomic::Ordering::Relaxed);
        return (keyword & keyword_any) != 0 && (keyword & keyword_all) == keyword_all;
    }

    // Unregister racing with event writes.
    //
    // `write_transfer` and `set_information` load `reg_handle` and then call into ETW. If
    // `unregister` runs between that load and the ETW call, the writer passes ETW a handle
    // that has already been unregistered.
    // - User mode: ntdll tolerates stale REGHANDLEs in EventWrite*/EventSetInformation
    //   (not well documented, but relied upon), so `unregister` is a safe fn.
    // - Kernel mode: the REGHANDLE refers to the kernel registration object, so this is
    //   a use-after-free. Like TraceLoggingProvider.h, we make this the caller's
    //   responsibility (with the `kernel_mode` feature, `unregister` is an `unsafe fn`
    //   with this requirement in its Safety contract) rather than paying for it on every
    //   write.
    //
    // Possible lock-free fix ("rundown protection"), which would allow `unregister` to be
    // a safe fn in kernel mode too (only needs to be compiled in for kernel_mode):
    // - Add `in_flight: atomic::AtomicUsize` to ProviderContextInner.
    // - Writers (`write_transfer`, `set_information`):
    //       in_flight.fetch_add(1, SeqCst);
    //       let h = reg_handle.load(SeqCst);
    //       if h != 0 { result = EventWriteTransfer(h, ...); }
    //       in_flight.fetch_sub(1, Release);
    // - `unregister` (while holding `busy`):
    //       let h = reg_handle.swap(0, SeqCst);
    //       if in_flight.load(SeqCst) != 0 {
    //           while in_flight.load(Acquire) != 0 { core::hint::spin_loop(); }
    //       }
    //       EventUnregister(h);
    // - The writer's fetch_add/load and unregister's swap/first load must all be SeqCst
    //   (Dekker-style store->load ordering): either unregister observes the writer's
    //   increment and waits for it, or the writer observes handle 0 and skips the call.
    //   The Release fetch_sub pairs with the Acquire spin load so the ETW call
    //   happens-before EventUnregister.
    // - The crate is no_std, so unregister must spin (no OS wait primitive).
    // - Cost: two RMWs on a shared (contended) cache line per written event, plus
    //   unregister may spin for the duration of in-flight writes.
    fn unregister(&self) -> u32 {
        let result;

        let was_busy = self.busy.swap(true, atomic::Ordering::Acquire);
        if was_busy {
            result = 0;
        } else {
            let reg_handle = self.reg_handle.swap(0, atomic::Ordering::AcqRel);
            if reg_handle == 0 {
                result = 0;
            } else {
                #[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
                {
                    result = unsafe { EventUnregister(reg_handle) };
                }
                #[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
                {
                    result = unsafe { EtwUnregister(reg_handle) };
                }
                self.level.store(-1, atomic::Ordering::Release);
            }

            self.busy.swap(false, atomic::Ordering::Release);
        }

        return result;
    }

    fn register(
        &self,
        provider_id: &Guid,
        callback_fn: Option<ProviderEnableCallback>,
        callback_context: usize,
        provider_traits: &[u8],
    ) -> u32 {
        let was_busy = self.busy.swap(true, atomic::Ordering::Acquire);
        if was_busy {
            panic!("provider.register called simultaneously with another call to register or unregister.");
        }

        if self.reg_handle.load(atomic::Ordering::Acquire) != 0 {
            self.busy.swap(false, atomic::Ordering::Relaxed);
            panic!("provider.register called when provider is already registered");
        }

        self.callback_fn.store(
            callback_fn.map_or(core::ptr::null_mut(), |callback| callback as *mut ()),
            atomic::Ordering::Release,
        );
        self.callback_context
            .store(callback_context, atomic::Ordering::Release);

        // Receive the handle in local storage and publish it with an atomic store after
        // EventRegister returns: other threads may be loading reg_handle concurrently, so
        // ETW must not write to it directly. ETW may invoke the enable callback (making
        // `enabled()` return true) before EventRegister returns; until the handle is
        // published, `write_transfer` sees handle 0 and skips the ETW call, so events
        // written during that window are dropped.
        let self_ptr = self as *const Self;
        let mut reg_handle = 0u64;
        #[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
        let result = unsafe {
            EventRegister(
                provider_id,
                Self::outer_callback,
                self_ptr as usize,
                &mut reg_handle,
            )
        };
        #[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
        let result = unsafe {
            EtwRegister(
                provider_id,
                Self::outer_callback,
                self_ptr as usize,
                &mut reg_handle,
            )
        };

        if result == 0 {
            if !provider_traits.is_empty() {
                // Set traits (e.g. provider name and group) before publishing the handle
                // so that no event is written before the traits are set. Failure is
                // ignored, as before: events are still written, just without traits.
                #[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
                unsafe {
                    EventSetInformation(
                        reg_handle,
                        2, // EventProviderSetTraits
                        provider_traits.as_ptr(),
                        provider_traits.len() as u32,
                    )
                };
                #[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
                unsafe {
                    EtwSetInformation(
                        reg_handle,
                        2, // EventProviderSetTraits
                        provider_traits.as_ptr(),
                        provider_traits.len() as u32,
                    )
                };
            }

            self.reg_handle.store(reg_handle, atomic::Ordering::Release);
        }
        self.busy.swap(false, atomic::Ordering::Release);

        return result;
    }

    #[cfg(all(windows, feature = "etw"))]
    fn outer_callback_impl(
        &self,
        source_id: &Guid,
        event_control_code: u32,
        level: u8,
        match_any_keyword: u64,
        match_all_keyword: u64,
        filter_data: usize,
    ) {
        match event_control_code {
            0 => {
                self.level.store(-1, atomic::Ordering::Release);
            }
            1 => {
                // Store keywords before level; see `enabled`.
                self.keyword_any
                    .store(match_any_keyword, atomic::Ordering::Relaxed);
                self.keyword_all
                    .store(match_all_keyword, atomic::Ordering::Relaxed);
                // ETW treats an enable level of 0 as "all levels".
                self.level.store(
                    if level == 0 { 255 } else { level as i32 },
                    atomic::Ordering::Release,
                );
            }
            _ => {}
        }

        let callback_ptr = self.callback_fn.load(atomic::Ordering::Acquire);
        if !callback_ptr.is_null() {
            // SAFETY: register() only stores null or a ProviderEnableCallback.
            let callback_fn =
                unsafe { core::mem::transmute::<*mut (), ProviderEnableCallback>(callback_ptr) };
            callback_fn(
                source_id,
                event_control_code,
                Level(level),
                match_any_keyword,
                match_all_keyword,
                filter_data,
                self.callback_context.load(atomic::Ordering::Acquire),
            );
        }
    }

    /// Implements the native ETW provider enable callback.
    unsafe extern "system" fn outer_callback(
        source_id: &Guid,
        event_control_code: u32,
        level: u8,
        match_any_keyword: u64,
        match_all_keyword: u64,
        filter_data: usize,
        outer_context: usize,
    ) {
        unsafe {
            (*(outer_context as *const Self)).outer_callback_impl(
                source_id,
                event_control_code,
                level,
                match_any_keyword,
                match_all_keyword,
                filter_data,
            );
        }
    }
}

#[cfg(all(windows, feature = "etw", not(feature = "kernel_mode")))]
#[link(name="OneCore_apiset")]
extern "system" {
    fn EventUnregister(reg_handle: u64) -> u32;
    fn EventRegister(
        provider_id: &Guid,
        outer_callback: OuterEnableCallback,
        outer_context: usize,
        reg_handle: &mut u64,
    ) -> u32;

    fn EventSetInformation(
        reg_handle: u64,
        information_class: u32,
        information: *const u8,
        information_length: u32,
    ) -> u32;
    fn EventWriteTransfer(
        reg_handle: u64,
        descriptor: &EventDescriptor,
        activity_id: Option<&[u8; 16]>,
        related_id: Option<&[u8; 16]>,
        data_count: u32,
        data: *const EventDataDescriptor,
    ) -> u32;
    fn EventActivityIdControl(control_code: u32, activity_id: &mut Guid) -> u32;
}

#[cfg(all(windows, feature = "etw", feature = "kernel_mode"))]
extern "system" {
    fn EtwUnregister (reg_handle: u64) -> u32;
    fn EtwRegister(
        provider_id: &Guid,
        outer_callback: OuterEnableCallback,
        outer_context: usize,
        reg_handle: &mut u64,
    ) -> u32;
    fn EtwSetInformation(
        reg_handle: u64,
        information_class: u32,
        information: *const u8,
        information_length: u32,
    ) -> u32;
    fn EtwWriteTransfer(
        reg_handle: u64,
        descriptor: &EventDescriptor,
        activity_id: Option<&[u8; 16]>,
        related_id: Option<&[u8; 16]>,
        data_count: u32,
        data: *const EventDataDescriptor,
    ) -> u32;
    fn EtwActivityIdControl(control_code: u32, activity_id: &mut Guid) -> u32;
}

#[cfg(all(test, windows, feature = "etw"))]
mod tests {
    use super::*;

    fn enable(context: &ProviderContext, event_control_code: u32, level: u8) {
        unsafe {
            ProviderContextInner::outer_callback(
                &Guid::zero(),
                event_control_code,
                level,
                0,
                0,
                0,
                &context.inner as *const ProviderContextInner as usize,
            );
        }
    }

    #[test]
    fn enable_level_0_enables_all_levels() {
        let context = ProviderContext::new();
        enable(&context, 1, 0);
        assert!(context.enabled(Level::LogAlways, 0));
        assert!(context.enabled(Level::Verbose, 0));
        assert!(context.enabled(Level(255), 0));

        enable(&context, 0, 0);
        assert!(!context.enabled(Level::LogAlways, 0));
    }

    #[test]
    fn enable_level_nonzero_filters_levels() {
        let context = ProviderContext::new();
        enable(&context, 1, 4);
        assert!(context.enabled(Level::Informational, 0));
        assert!(!context.enabled(Level::Verbose, 0));
    }
}
