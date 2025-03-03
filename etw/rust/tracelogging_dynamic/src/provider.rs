// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use alloc::vec::Vec;
use core::fmt;
use core::pin::Pin;
use core::str::from_utf8;

use tracelogging::Guid;
use tracelogging::Level;
use tracelogging::ProviderEnableCallback;
use tracelogging::_internal::ProviderContext;

#[allow(unused_imports)] // For docs
use crate::EventBuilder;

/// Represents a connection for writing dynamic TraceLogging (manifest-free) events to
/// ETW.
///
/// # Overview
///
/// - Create a pinned Provider object, e.g.
///   - `let provider = pin!(Provider::new("MyCompany.MyComponent", &tld::Provider::options()));`
///   - `let provider = Box::pin(Provider::new("MyCompany.MyComponent", &tld::Provider::options()));`
///   - The provider name should be short and human-readable but should be unique enough
///     to not collide with the names of other providers. You'll typically want to
///     include a company or organization in the provider name to ensure uniqueness. For
///     example, `"MyCompany.MyComponent"`.
///   - You'll usually just use default `&Provider::options()` for the options parameter.
///     For advanced scenarios, you can use the options parameter to specify a provider
///     group id or a provider enable callback.
/// - Call `unsafe { provider.as_ref().register(); }` to open the connection to ETW.
///   - `register` is unsafe because a registered provider **must** be properly
///     unregistered. This automatically happens when the provider is dropped so this
///     normally isn't an issue unless the provider is declared as a static variable.
///   - Since the `register` method requires a `Pin`, you'll usually need to use `as_ref()`
///     when calling it.
/// - Use an [EventBuilder] to construct and write events as needed.
/// - The provider will automatically unregister when it is dropped. You can manually call
///   `unregister()` if you want to unregister sooner or if the provider is static.
///
/// # Pinning your provider
///
/// Since the provider manages an asynchronous ETW callback, it must be pinned before you
/// can call register and must remain pinned until it is dropped. Some ways to do this:
///
/// ## On the heap (safe pinning):
///
/// ```
/// use tracelogging_dynamic as tld;
/// let provider = Box::pin(tld::Provider::new("MyCompany.MyComponent", &tld::Provider::options()));
/// unsafe {
///     provider.as_ref().register();
/// }
/// ```
///
/// ## On the stack (safe pinning):
///
/// ```
/// use core::pin::pin;
/// use tracelogging_dynamic as tld;
/// let provider = pin!(tld::Provider::new("MyCompany.MyComponent", &tld::Provider::options()));
/// unsafe {
///     provider.as_ref().register();
/// }
/// ```
///
/// ## As a field on your own struct (unsafe pinning):
///
/// You'll need to pin your own struct and then you can use your struct's pin to get a
/// pinned provider.
///
/// ```
/// use tracelogging_dynamic as tld;
/// # use core::pin::Pin;
/// # struct MyPinnableStruct { provider: tld::Provider };
/// # let mut my_pinned_struct = MyPinnableStruct { provider: tld::Provider::new("MyCompany.MyComponent", &tld::Provider::options()) };
/// # let mut my_pinned_struct = unsafe { Pin::new_unchecked(&my_pinned_struct) };
/// let provider = unsafe { my_pinned_struct.map_unchecked(|s| &s.provider) };
/// unsafe {
///     provider.as_ref().register();
/// }
/// ```
pub struct Provider {
    pub(crate) context: ProviderContext,
    pub(crate) meta: Vec<u8>, // provider metadata
    id: Guid,
    callback_fn: Option<ProviderEnableCallback>,
    callback_context: usize,
}

impl Provider {
    /// Returns the current thread's thread-local activity id.
    /// (Calls
    /// [EventActivityIdControl](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventactivityidcontrol)
    /// with control code `EVENT_ACTIVITY_CTRL_GET_ID`.)
    ///
    /// The thread-local activity id will be used if [EventBuilder::write] is called with
    /// an `activity_id` of `None`.
    pub fn current_thread_activity_id() -> Guid {
        let mut activity_id = Guid::zero();
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
    /// The thread-local activity id will be used if [EventBuilder::write] is called with
    /// an `activity_id` of `None`.
    ///
    /// Important: thread-local activity id should follow scoping rules. If you set the
    /// thread-local activity id in a scope, you should restore the previous value before
    /// exiting the scope.
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
    /// returned value is locally-unique: it is guaranteed to be unique from all other
    /// values generated by `create_activity_id` in the current boot session.
    ///
    /// Activity ids need to be unique within the trace but do not need to be
    /// globally-unique, so your activity ids can use either a real GUID/UUID or a
    /// locally-unique id generated by `create_activity_id`. Use `create_activity_id` to
    /// generate locally-unique activity ids or use [Guid::new] to generate
    /// globally-unique activity ids.
    pub fn create_activity_id() -> Guid {
        let mut activity_id = Guid::zero();
        ProviderContext::activity_id_control(
            3, // CreateId
            &mut activity_id,
        );
        return activity_id;
    }

    /// Returns a GUID generated from a case-insensitive hash of the specified
    /// trace provider name using the same algorithm as is used by many other ETW
    /// APIs and tools. Given the same name, it will always generate the same
    /// GUID. (Same as [Guid::from_name].)
    ///
    /// The result can (and usually should) be used as the provider id.
    ///
    /// **Note:** Do not use guid_from_name to generate activity ids.
    /// ```
    /// use tracelogging_dynamic as tld;
    /// assert_eq!(
    ///    tld::Provider::guid_from_name("MyProvider"),
    ///    tld::Guid::from_u128(&0xb3864c38_4273_58c5_545b_8b3608343471));
    /// ```
    pub fn guid_from_name(name: &str) -> Guid {
        return Guid::from_name(name);
    }

    /// Returns a default ProviderOptions.
    pub fn options() -> ProviderOptions {
        return ProviderOptions::new();
    }

    /// Creates a new unregistered provider using `Guid::from_name(name)` as the provider
    /// id.
    ///
    /// Use `register()` to register the provider. If the provider is not registered,
    /// `enabled()` will return false and `EventBuilder::write()` will be a no-op.
    ///
    /// `name` must be less than 32KB and must not contain `'\0'`. It should be short,
    /// human-readable, and unique enough to not conflict with names of other providers.
    /// The provider name will typically include a company name and a component name,
    /// e.g. "MyCompany.MyComponent".
    ///
    /// `options` can usually be `&Provider::options()`. If the provider needs to
    /// join a provider group, use `Provider::options().group_id(provider_group_id)`.
    /// If the provider needs to specify a custom provider enable callback, use
    /// `Provider::options().callback(callback_fn, callback_context)`.
    pub fn new(name: &str, options: &ProviderOptions) -> Self {
        return Self::new_with_id(name, options, &Guid::from_name(name));
    }

    /// Creates a new unregistered provider using the specified custom provider id.
    ///
    /// Use `register()` to register the provider. If the provider is not registered,
    /// `enabled()` will return false and `EventBuilder::write()` will be a no-op.
    ///
    /// `name` must be less than 32KB and must not contain `'\0'`. It should be short,
    /// human-readable, and unique enough to not conflict with names of other providers.
    /// The provider name will typically include a company name and a component name,
    /// e.g. "MyCompany.MyComponent".
    ///
    /// `options` can usually be `&Provider::options()`. If the provider needs to
    /// join a provider group, use `Provider::options().group_id(provider_group_id)`.
    /// If the provider needs to specify a custom provider enable callback, use
    /// `Provider::options().callback(callback_fn, callback_context)`.
    ///
    /// `id` is the provider id. Since the provider id and the provider name are tightly
    /// coupled, the provider id should usually be generated from the name using
    /// `Guid::from_name(name)`.
    pub fn new_with_id(name: &str, options: &ProviderOptions, id: &Guid) -> Self {
        assert!(
            name.len() < 32768,
            "provider name.len() must be less than 32KB"
        );
        debug_assert!(!name.contains('\0'), "provider name must not contain '\\0'");

        const GROUP_TRAIT_LEN: u16 = 2 + 1 + 16;
        let name_len = name.len() as u16;
        let traits_len = if options.group_id.is_some() {
            GROUP_TRAIT_LEN
        } else {
            0
        };
        let meta_len = 2 + name_len + 1 + traits_len;
        let mut meta = Vec::with_capacity(meta_len as usize);

        meta.extend_from_slice(&meta_len.to_le_bytes());
        meta.extend_from_slice(name.as_bytes());
        meta.push(0);

        if traits_len != 0 {
            meta.extend_from_slice(&GROUP_TRAIT_LEN.to_le_bytes());
            meta.push(1); // EtwProviderTraitTypeGroup
            meta.extend_from_slice(&options.group_id.unwrap().to_bytes_le());
        }

        debug_assert_eq!(
            meta.len(),
            meta_len as usize,
            "Bug: Incorrect meta length reservation"
        );

        return Self {
            context: ProviderContext::new(),
            meta,
            id: *id,
            callback_fn: options.callback_fn,
            callback_context: options.callback_context,
        };
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
    ///
    /// Use `provider.unregister()` if you want to unregister the provider before it goes
    /// out of scope. The provider automatically unregisters when it is dropped so most
    /// users do  not need to call `unregister` directly.
    pub fn unregister(&self) -> u32 {
        return self.context.unregister();
    }

    /// Registers the provider, connecting it to the Windows ETW system.
    ///
    /// This method will panic if the provider is already registered. You must call
    /// [Provider::unregister()] before you can re-register a provider.
    ///
    /// Since the provider manages an ETW callback, it must be pinned before you can call
    /// `register`. Refer to the documentation for [Provider] for examples of how to pin
    /// the provider. Once you have a pinned `provider`, you'll use
    /// `provider.as_mut().register(...)` to register it.
    ///
    /// Returns 0 for success or a Win32 error from
    /// [EventRegister](https://docs.microsoft.com/windows/win32/api/evntprov/nf-evntprov-eventregister)
    /// for failure. The return value is for diagnostic purposes only and should
    /// generally be ignored in retail builds: if `register` fails then
    /// [Provider::enabled()] will always return `false`, [Provider::unregister()] will
    /// be a no-op, and [EventBuilder::write()] will be a no-op.
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
    /// - If the provider is in a DLL, it **must** be unregistered before the DLL unloads.
    ///
    ///   [Provider::unregister] is called automatically when the provider is dropped so
    ///   this generally isn't an issue unless the provider is declared as a static variable.
    ///
    ///   If a provider variable is registered by a DLL and the DLL unloads without dropping
    ///   the variable, the process may subsequently crash. This occurs because `register`
    ///   enables an ETW callback into the calling DLL and `unregister` ensures that the
    ///   callback is disabled. If the module unloads without disabling the callback, the
    ///   process will crash the next time that ETW tries to invoke the callback.
    ///
    /// - Once registered, you may not move-out of the provider variable until the
    ///   provider is dropped. (This is implied by the rules for `Pin` but repeated here
    ///   for clarity.)
    pub unsafe fn register(self: Pin<&Self>) -> u32 {
        let result = unsafe { self
            .context
            .register(&self.id, self.callback_fn, self.callback_context) };
        if result == 0 {
            self.context.set_information(
                2, // EventProviderSetTraits
                &self.meta[..],
            );
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

/// Builder for advanced provider configuration. Used when registering a provider.
///
/// In most cases, you'll just use the default options.
/// ```
/// # use core::pin::Pin;
/// # use tracelogging_dynamic as tld;
/// let mut provider = Box::pin(tld::Provider::new(
///     "MyCompany.MyComponent",
///     &tld::Provider::options()));
/// unsafe {
///     provider.as_ref().register();
/// }
/// ```
///
/// If your provider needs to join a provider group, use `group_id(provider_group_id)`.
/// ```
/// # use core::pin::Pin;
/// # use tracelogging_dynamic as tld;
/// let provider = Box::pin(tld::Provider::new(
///     "MyCompany.MyComponent",
///     tld::Provider::options().group_id(&tld::Guid::from_name("MyCompany.GroupName"))));
/// unsafe {
///     provider.as_ref().register();
/// }
/// ```
///
/// If your provider needs a custom provider enable callback, use `callback(fn, ctx)`.
/// ```
/// # use core::pin::Pin;
/// # use tracelogging_dynamic as tld;
/// fn my_callback(
///     _source_id: &tld::Guid,
///     _event_control_code: u32,
///     _level: tld::Level,
///     _match_any_keyword: u64,
///     _match_all_keyword: u64,
///     _filter_data: usize,
///     callback_context: usize) {
///
///     // Do stuff...
///     assert_eq!(callback_context, 0xDEADBEEF);
/// }
///
/// let provider = Box::pin(tld::Provider::new(
///     "MyCompany.MyComponent",
///     tld::Provider::options().callback(my_callback, 0xDEADBEEF)));
/// unsafe {
///     provider.as_ref().register();
/// }
/// ```
#[derive(Clone, Copy, Default)]
pub struct ProviderOptions {
    group_id: Option<Guid>,
    callback_fn: Option<ProviderEnableCallback>,
    callback_context: usize,
}

impl ProviderOptions {
    /// Creates default provider options.
    /// - No provider group id.
    /// - No enable callback function or callback context.
    pub const fn new() -> Self {
        return Self {
            group_id: None,
            callback_fn: None,
            callback_context: 0,
        };
    }

    /// Sets the id of the
    /// [provider group](https://docs.microsoft.com/windows/win32/etw/provider-traits)
    /// that the provider should join.
    ///
    /// Most providers do not join any provider group so this is usually not called.
    pub fn group_id(&mut self, value: &Guid) -> &mut Self {
        self.group_id = Some(*value);
        return self;
    }

    /// Sets a custom
    /// [provider enable callback](https://docs.microsoft.com/windows/win32/api/evntprov/nc-evntprov-penablecallback)
    /// and context.
    ///
    /// Most providers do not need a custom provider enable callback so this is usually
    /// not called.
    pub fn callback(
        &mut self,
        callback_fn: ProviderEnableCallback,
        callback_context: usize,
    ) -> &mut Self {
        self.callback_fn = Some(callback_fn);
        self.callback_context = callback_context;
        return self;
    }
}

impl fmt::Debug for ProviderOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let callback_ptr = match self.callback_fn {
            None => core::ptr::null(),
            Some(p) => p as *const (),
        };
        return write!(
            f,
            "ProviderOptions {{ group_id: \"{:?}\", callback_fn: {:?}, callback_context: {:x} }}",
            self.group_id, callback_ptr, self.callback_context
        );
    }
}
