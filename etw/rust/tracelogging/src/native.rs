use core::marker::PhantomPinned;

#[cfg(all(windows, not(feature = "no_windows")))]
use core::cell::UnsafeCell;
#[cfg(all(windows, not(feature = "no_windows")))]
use core::sync::atomic;

use crate::descriptors::EventDataDescriptor;
use crate::descriptors::EventDescriptor;
use crate::enums::Level;
use crate::guid::Guid;

/// Possible configurations under which this crate can be compiled.
pub enum NativeImplementation {
    /// Crate compiled for other configuration.
    Other,
    /// Crate compiled for Windows (ETW) configuration.
    Windows,
}

/// The configuration under which this crate was compiled.
pub const NATIVE_IMPLEMENTATION: NativeImplementation =
    if cfg!(all(windows, not(feature = "no_windows"))) {
        NativeImplementation::Windows
    } else {
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

#[cfg(all(windows, not(feature = "no_windows")))]
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

    #[cfg(all(windows, not(feature = "no_windows")))]
    cell: UnsafeCell<ProviderContextInner>,
}

impl ProviderContext {
    /// Windows: return EventActivityIdControl(...);
    /// Other: return ERROR_NOT_SUPPORTED;
    pub fn activity_id_control(_control_code: u32, _activity_id: &mut Guid) -> u32 {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = 50; // ERROR_NOT_SUPPORTED
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
        {
            result = unsafe { EventActivityIdControl(_control_code, _activity_id) };
        }
        return result;
    }

    /// Creates a new provider context.
    pub const fn new() -> ProviderContext {
        return ProviderContext {
            _pinned: PhantomPinned,

            #[cfg(all(windows, not(feature = "no_windows")))]
            cell: UnsafeCell::new(ProviderContextInner::new()),
        };
    }

    /// Returns the registration handle. For diagnostic purposes only.
    pub const fn reg_handle(&self) -> u64 {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = 0;
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
        {
            let inner_ptr: *const ProviderContextInner = self.cell.get();
            let inner = unsafe { &*inner_ptr };
            result = inner.reg_handle;
        }
        return result;
    }

    /// Returns true if the provider is enabled at the specified level and keyword.
    #[inline(always)]
    pub const fn enabled(&self, _level: Level, _keyword: u64) -> bool {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = false;
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
        {
            let inner_ptr: *const ProviderContextInner = self.cell.get();
            let inner = unsafe { &*inner_ptr };
            result = (_level.0 as i32) <= inner.level && inner.enabled_keyword(_keyword);
        }
        return result;
    }

    /// Calls EventUnregister and sets reg_handle = 0.
    ///
    /// ## Preconditions
    /// - This will panic if it overlaps with another thread simultaneously calling
    ///   register or unregister.
    pub fn unregister(&self) -> u32 {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = 0;
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
        {
            let inner_ptr: *mut ProviderContextInner = self.cell.get();
            let inner_mut = unsafe { &mut *inner_ptr };
            result = inner_mut.unregister();
        }
        return result;
    }

    /// Calls EventRegister.
    ///
    /// ## Preconditions
    /// - This will panic if provider is currently registered.
    /// - This will panic if it overlaps with another thread simultaneously calling
    ///   register or unregister.
    ///
    /// ## Safety
    /// 1. Pinning: Context must not be moved-from as long as provider is registered.
    /// 2. Unregister: If a provider is registered in a DLL, unregister **must** be
    ///    called before the DLL unloads.
    pub unsafe fn register(
        &self,
        _provider_id: &Guid,
        _callback_fn: Option<ProviderEnableCallback>,
        _callback_context: usize,
    ) -> u32 {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = 0;
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
        {
            result = /* unsafe */ { &mut *self.cell.get() }.register(
                _provider_id,
                _callback_fn,
                _callback_context);
        }
        return result;
    }

    /// Calls EventSetInformation.
    pub fn set_information(&self, _information_class: u32, _information: &[u8]) -> u32 {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = 0;
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
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
        return result;
    }

    /// Calls EventWriteTransfer.
    pub fn write_transfer(
        &self,
        _descriptor: &EventDescriptor,
        _activity_id: Option<&Guid>,
        _related_id: Option<&Guid>,
        _data: &[EventDataDescriptor],
    ) -> u32 {
        let result;
        #[cfg(not(all(windows, not(feature = "no_windows"))))]
        {
            result = 0;
        }
        #[cfg(all(windows, not(feature = "no_windows")))]
        {
            result = unsafe {
                EventWriteTransfer(
                    self.reg_handle(),
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

unsafe impl Sync for ProviderContext {}

impl Default for ProviderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProviderContext {
    /// Calls unregister.
    fn drop(&mut self) {
        self.unregister();
    }
}

#[cfg(all(windows, not(feature = "no_windows")))]
struct ProviderContextInner {
    level: i32, // -1 means not enabled by anybody.
    busy: atomic::AtomicBool,
    reg_handle: u64,
    keyword_any: u64,
    keyword_all: u64,
    callback_fn: Option<ProviderEnableCallback>,
    callback_context: usize,
}

#[cfg(all(windows, not(feature = "no_windows")))]
impl ProviderContextInner {
    const fn new() -> Self {
        return Self {
            level: -1,
            busy: atomic::AtomicBool::new(false),
            reg_handle: 0,
            keyword_any: 0,
            keyword_all: 0,
            callback_fn: None,
            callback_context: 0,
        };
    }

    /// Returns true if the provider is enabled at the specified keyword.
    const fn enabled_keyword(&self, keyword: u64) -> bool {
        return keyword == 0
            || ((keyword & self.keyword_any) != 0
                && (keyword & self.keyword_all) == self.keyword_all);
    }

    fn unregister(&mut self) -> u32 {
        let was_busy = self.busy.swap(true, atomic::Ordering::Acquire);
        if was_busy {
            panic!("provider.unregister called simultaneously with another call to register or unregister.");
        }

        let result;
        if self.reg_handle == 0 {
            result = 0;
        } else {
            result = unsafe { EventUnregister(self.reg_handle) };
            self.level = -1;
            self.reg_handle = 0;
        }

        self.busy.swap(false, atomic::Ordering::Release);
        return result;
    }

    fn register(
        &mut self,
        provider_id: &Guid,
        callback_fn: Option<ProviderEnableCallback>,
        callback_context: usize,
    ) -> u32 {
        let was_busy = self.busy.swap(true, atomic::Ordering::Acquire);
        if was_busy {
            panic!("provider.register called simultaneously with another call to register or unregister.");
        }

        if self.reg_handle != 0 {
            self.busy.swap(false, atomic::Ordering::Relaxed);
            panic!("provider.register called when provider is already registered");
        }

        self.callback_fn = callback_fn;
        self.callback_context = callback_context;

        let self_ptr: *mut Self = self;
        let result = unsafe {
            EventRegister(
                provider_id,
                Self::outer_callback,
                self_ptr as usize,
                &mut self.reg_handle,
            )
        };

        self.busy.swap(false, atomic::Ordering::Release);

        return result;
    }

    #[cfg(all(windows, not(feature = "no_windows")))]
    fn outer_callback_impl(
        &mut self,
        source_id: &Guid,
        event_control_code: u32,
        level: u8,
        match_any_keyword: u64,
        match_all_keyword: u64,
        filter_data: usize,
    ) {
        match event_control_code {
            0 => {
                self.level = -1;
            }
            1 => {
                self.level = level as i32;
                self.keyword_any = match_any_keyword;
                self.keyword_all = match_all_keyword;
            }
            _ => {}
        }

        if let Some(callback_fn) = self.callback_fn {
            callback_fn(
                source_id,
                event_control_code,
                Level(level),
                match_any_keyword,
                match_all_keyword,
                filter_data,
                self.callback_context,
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
        (&mut *(outer_context as *mut Self)).outer_callback_impl(
            source_id,
            event_control_code,
            level,
            match_any_keyword,
            match_all_keyword,
            filter_data,
        );
    }
}

#[cfg(all(windows, not(feature = "no_windows")))]
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
        activity_id: Option<&Guid>,
        related_id: Option<&Guid>,
        data_count: u32,
        data: *const EventDataDescriptor,
    ) -> u32;
    fn EventActivityIdControl(control_code: u32, activity_id: &mut Guid) -> u32;
}
