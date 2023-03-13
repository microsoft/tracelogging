// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use core::pin::Pin;
use tracelogging_dynamic::*;

#[test]
fn provider() {
    assert_eq!(
        Provider::guid_from_name("MyProviderName"),
        Guid::from_u128(&0xb7aa4d18_240c_5f41_5852_817dbf477472)
    );

    assert_eq!(
        win_filetime_from_systemtime!(std::time::SystemTime::UNIX_EPOCH),
        0x19DB1DED53E8000
    );

    let new_aid1 = Provider::create_activity_id();
    let new_aid2 = Provider::create_activity_id();

    if let NativeImplementation::Windows = NATIVE_IMPLEMENTATION {
        assert_ne!(new_aid1, Guid::zero());
        assert_ne!(new_aid2, Guid::zero());
        assert_ne!(new_aid1, new_aid2);
    }

    let aid0 = Provider::current_thread_activity_id();
    assert_eq!(aid0, Provider::set_current_thread_activity_id(&new_aid1));

    let aid1 = Provider::current_thread_activity_id();
    assert_eq!(new_aid1, aid1);
    assert_eq!(aid1, Provider::set_current_thread_activity_id(&new_aid2));

    let aid2 = Provider::current_thread_activity_id();
    assert_eq!(new_aid2, aid2);
    assert_eq!(aid2, Provider::set_current_thread_activity_id(&aid0));

    assert_eq!(Guid::from_name("Hello"), Provider::guid_from_name("Hello"));

    fn my_callback(
        _source_id: &Guid,
        _event_control_code: u32,
        _level: Level,
        _match_any_keyword: u64,
        _match_all_keyword: u64,
        _filter_data: usize,
        callback_context: usize,
    ) {
        assert_eq!(callback_context, 0xDEADBEEF);
    }

    println!(
        "{:?}",
        Provider::options()
            .group_id(&Guid::zero())
            .callback(my_callback, 1)
            .group_id(&Guid::zero())
    );

    let mut provider = Box::pin(Provider::new());
    assert_eq!(provider.name(), "");
    assert_eq!(provider.id(), &Guid::zero());

    unsafe {
        provider.as_mut().register(
            "MyCompany.MyComponent",
            Provider::options().group_id(&Guid::zero()),
        )
    };
    assert_eq!(provider.name(), "MyCompany.MyComponent");
    assert_eq!(provider.id(), &Guid::from_name("MyCompany.MyComponent"));

    let mut provider = Provider::new(); // Temporary that will be shadowed.
    let mut provider = unsafe { Pin::new_unchecked(&mut provider) };
    assert_eq!(provider.name(), "");
    assert_eq!(provider.id(), &Guid::zero());

    unsafe {
        provider
            .as_mut()
            .register_with_id("Hello", &Provider::options(), &aid1)
    };
    assert_eq!(provider.name(), "Hello");
    assert_eq!(provider.id(), &aid1);

    provider.unregister();

    //let mut provider = core::pin::pin!(Provider::new());
    //provider.as_mut().register("MyCompany.MyComponent", &Provider::options());

    struct HasProvider {
        provider: Provider,
    }

    let mut my_pinned_struct = HasProvider {
        provider: Provider::new(),
    };
    let my_pinned_struct = unsafe { Pin::new_unchecked(&mut my_pinned_struct) };

    let mut provider = unsafe { my_pinned_struct.map_unchecked_mut(|s| &mut s.provider) };
    unsafe {
        provider.as_mut().register(
            "MyCompany.MyComponent",
            Provider::options().callback(my_callback, 0xDEADBEEF),
        )
    };

    _ = provider.enabled(Level::Verbose, 0x123);

    provider.unregister();

    let mut b = EventBuilder::new();
    let aid = Provider::create_activity_id();
    let rid = Provider::current_thread_activity_id();

    unsafe {
        provider.as_mut().register(
            "TraceLoggingDynamicTest",
            Provider::options()
                .callback(my_callback, 0xDEADBEEF)
                .group_id(&Guid::from_name("TraceLoggingDynamicTestGroup")),
        )
    };
    b.reset("GroupEvent-Start", Level::Verbose, 0x1, 0)
        .opcode(Opcode::Start)
        .write(&provider, Some(&aid), Some(&rid));
    b.reset("GroupEvent-Stop", Level::Verbose, 0x1, 0)
        .opcode(Opcode::Stop)
        .write(&provider, Some(&aid), None);
}

#[test]
fn builder() {
    let mut p = Provider::new(); // Temporary that will be shadowed.
    let mut p = unsafe { Pin::new_unchecked(&mut p) };
    let mut b = EventBuilder::new();
    println!("{:?}", b);

    unsafe {
        p.as_mut()
            .register("TraceLoggingDynamicTest", &Provider::options())
    };

    b.reset("Default", Level::Verbose, 0x1, 0)
        .write(&p, None, None);

    b.reset("4v2o6t123c0l3k11", Level::Warning, 0x11, 0)
        .id_version(4, 2)
        .channel(Channel::TraceClassic)
        .opcode(Opcode::Reply)
        .task(123)
        .write(&p, None, None);

    b.reset("tag0xFE00000", Level::Verbose, 0x1, 0xFE00000)
        .write(&p, None, None);
    b.reset("tag0xFEDC000", Level::Verbose, 0x1, 0xFEDC000)
        .write(&p, None, None);
    b.reset("tag0xFEDCBAF", Level::Verbose, 0x1, 0xFEDCBAF)
        .write(&p, None, None);

    b.reset("fieldtag", Level::Verbose, 0x1, 0)
        .add_u8("0xFE00000", 0, OutType::Default, 0xFE00000)
        .add_u8("0xFEDC000", 0, OutType::Default, 0xFEDC000)
        .add_u8("0xFEDCBAF", 0, OutType::Default, 0xFEDCBAF)
        .write(&p, None, None);

    b.reset("outtypes", Level::Verbose, 0x1, 0)
        .add_u8("default100", 100, OutType::Default, 0)
        .add_u8("string65", 65, OutType::String, 0)
        .add_u8("bool1", 1, OutType::Boolean, 0)
        .add_u8("hex100", 100, OutType::Hex, 0)
        .write(&p, None, None);

    b.reset("structs", Level::Verbose, 0x1, 0)
        .add_u8("start", 0, OutType::Default, 0)
        .add_struct("struct1", 2, 0xFE00000)
        .add_u8("nested1", 1, OutType::Default, 0)
        .add_struct("struct2", 1, 0)
        .add_u8("nested2", 2, OutType::Default, 0)
        .write(&p, None, None);

    b.reset("cstrs-L4-kFF", Level::Informational, 0xff, 0)
        .add_u8("A", 65, OutType::String, 0)
        .add_cstr16("cstr16-", to_utf16("").as_slice(), OutType::Default, 0)
        .add_cstr16("cstr16-a", to_utf16("a").as_slice(), OutType::Default, 0)
        .add_cstr16("cstr16-0", to_utf16("\0").as_slice(), OutType::Default, 0)
        .add_cstr16("cstr16-a0", to_utf16("a\0").as_slice(), OutType::Default, 0)
        .add_cstr16("cstr16-0a", to_utf16("\0a").as_slice(), OutType::Default, 0)
        .add_cstr16(
            "cstr16-a0a",
            to_utf16("a\0a").as_slice(),
            OutType::Default,
            0,
        )
        .add_cstr8("cstr8-", "".as_bytes(), OutType::Default, 0)
        .add_cstr8("cstr8-a", "a".as_bytes(), OutType::Default, 0)
        .add_cstr8("cstr8-0", "\0".as_bytes(), OutType::Default, 0)
        .add_cstr8("cstr8-a0", "a\0".as_bytes(), OutType::Default, 0)
        .add_cstr8("cstr8-0a", "\0a".as_bytes(), OutType::Default, 0)
        .add_cstr8("cstr8-a0a", "a\0a".as_bytes(), OutType::Default, 0)
        .add_u8("A", 65, OutType::String, 0)
        .write(&p, None, None);

    validate(
        &p,
        &mut b,
        "UnicodeString",
        to_utf16("zutf16").as_slice(),
        |b, n, v, o, t| {
            b.add_cstr16(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_cstr16_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "AnsiString",
        "zutf8".as_bytes(),
        |b, n, v, o, t| {
            b.add_cstr8(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_cstr8_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Int8",
        -8,
        |b, n, v, o, t| {
            b.add_i8(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_i8_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "UInt8",
        8,
        |b, n, v, o, t| {
            b.add_u8(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_u8_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Int16",
        -16,
        |b, n, v, o, t| {
            b.add_i16(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_i16_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "UInt16",
        16,
        |b, n, v, o, t| {
            b.add_u16(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_u16_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Int32",
        -32,
        |b, n, v, o, t| {
            b.add_i32(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_i32_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "UInt32",
        32,
        |b, n, v, o, t| {
            b.add_u32(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_u32_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Int64",
        -64,
        |b, n, v, o, t| {
            b.add_i64(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_i64_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "UInt64",
        64,
        |b, n, v, o, t| {
            b.add_u64(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_u64_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "IntPtr",
        -3264,
        |b, n, v, o, t| {
            b.add_isize(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_isize_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "UIntPtr",
        3264,
        |b, n, v, o, t| {
            b.add_usize(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_usize_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Float32",
        3.2,
        |b, n, v, o, t| {
            b.add_f32(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_f32_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Float64",
        6.4,
        |b, n, v, o, t| {
            b.add_f64(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_f64_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Bool32",
        0,
        |b, n, v, o, t| {
            b.add_bool32(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_bool32_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Bool32",
        1,
        |b, n, v, o, t| {
            b.add_bool32(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_bool32_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Binary",
        "0123".as_bytes(),
        |b, n, v, o, t| {
            b.add_binary(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_binaryc_sequence(n, v, o, t);
        },
    ); // There is no add_binary_sequence.
    validate(
        &p,
        &mut b,
        "Guid",
        Provider::guid_from_name("sample"),
        |b, n, v, o, t| {
            b.add_guid(n, &v, o, t);
        },
        |b, n, v, o, t| {
            b.add_guid_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "FileTime",
        0x01d7ace794497cb5,
        |b, n, v, o, t| {
            b.add_filetime(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_filetime_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "SystemTime",
        [2022, 1, 1, 2, 3, 4, 5, 6],
        |b, n, v, o, t| {
            b.add_systemtime(n, &v, o, t);
        },
        |b, n, v, o, t| {
            b.add_systemtime_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "Sid",
        &[1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0][..],
        |b, n, v, o, t| {
            b.add_sid(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_sid_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "HexInt32",
        0xdeadbeef,
        |b, n, v, o, t| {
            b.add_hex32(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_hex32_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "HexInt64",
        0xdeadbeeffeeef000,
        |b, n, v, o, t| {
            b.add_hex64(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_hex64_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "HexIntPtr",
        0x1234,
        |b, n, v, o, t| {
            b.add_hexsize(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_hexsize_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "CountedString",
        to_utf16("utf16").as_slice(),
        |b, n, v, o, t| {
            b.add_str16(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_str16_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "CountedAnsiString",
        "utf8".as_bytes(),
        |b, n, v, o, t| {
            b.add_str8(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_str8_sequence(n, v, o, t);
        },
    );
    validate(
        &p,
        &mut b,
        "CountedBinary",
        "0123".as_bytes(),
        |b, n, v, o, t| {
            b.add_binaryc(n, v, o, t);
        },
        |b, n, v, o, t| {
            b.add_binaryc_sequence(n, v, o, t);
        },
    );
}

fn to_utf16(s: &str) -> Vec<u16> {
    Vec::from_iter(s.encode_utf16())
}

fn validate<T: Copy>(
    p: &Provider,
    b: &mut EventBuilder,
    name: &str,
    value: T,
    scalar: impl Fn(&mut EventBuilder, &str, T, OutType, u32),
    array: impl Fn(&mut EventBuilder, &str, &[T], OutType, u32),
) {
    let values = [value, value];
    b.reset(name, Level::Verbose, 0x1, 0);
    b.add_u8("A", b'A', OutType::String, 0);
    scalar(b, "scalar", value, OutType::Default, 0);
    array(b, "a0", &values[0..0], OutType::Default, 0);
    array(b, "a1", &values[0..1], OutType::Default, 0);
    array(b, "a2", &values, OutType::Default, 0);
    b.add_u8("A", b'A', OutType::String, 0);
    b.write(p, None, None);
}
