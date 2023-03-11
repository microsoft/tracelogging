// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![allow(clippy::needless_return)]

use tracelogging as tlg;
use tracelogging::_internal as tli;

#[allow(clippy::unusual_byte_groupings)]
#[test]
fn guid() {
    use std::format;
    use tlg::Guid;
    use uuid::Uuid;
    use windows::core::GUID;

    let nil = Guid::from_u128(&0);
    let myprovider = Guid::from_u128(&0xb3864c38_4273_58c5_545b_8b3608343471);
    let a3a2a1a0 = Guid::from_u128(&0xa3a2a1a0_b1b0_c1c0_d7d6_d5d4d3d2d1d0);

    assert_eq!(Guid::default(), nil);

    assert_eq!(Guid::from_name("myprovider"), myprovider);
    assert_eq!(Guid::from_name("MYPROVIDER"), myprovider);
    assert_eq!(
        Guid::from_fields(
            0xa3a2a1a0,
            0xb1b0,
            0xc1c0,
            [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]
        ),
        a3a2a1a0
    );
    assert_eq!(
        Guid::from_bytes_be(&[
            0xa3, 0xa2, 0xa1, 0xa0, 0xb1, 0xb0, 0xc1, 0xc0, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2,
            0xd1, 0xd0
        ]),
        a3a2a1a0
    );
    assert_eq!(
        Guid::from_bytes_le(&[
            0xa0, 0xa1, 0xa2, 0xa3, 0xb0, 0xb1, 0xc0, 0xc1, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2,
            0xd1, 0xd0
        ]),
        a3a2a1a0
    );
    assert_eq!(
        Guid::from_u128(&0xa3a2a1a0_b1b0_c1c0_d7d6d5d4d3d2d1d0),
        a3a2a1a0
    );
    assert_eq!(
        a3a2a1a0.to_fields(),
        (
            0xa3a2a1a0,
            0xb1b0,
            0xc1c0,
            [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]
        )
    );
    assert_eq!(
        a3a2a1a0.to_bytes_be(),
        [
            0xa3, 0xa2, 0xa1, 0xa0, 0xb1, 0xb0, 0xc1, 0xc0, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2,
            0xd1, 0xd0
        ]
    );
    assert_eq!(
        a3a2a1a0.to_bytes_le(),
        [
            0xa0, 0xa1, 0xa2, 0xa3, 0xb0, 0xb1, 0xc0, 0xc1, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2,
            0xd1, 0xd0
        ]
    );
    assert_eq!(a3a2a1a0.to_u128(), 0xa3a2a1a0_b1b0_c1c0_d7d6d5d4d3d2d1d0);
    assert_eq!(
        a3a2a1a0.to_utf8_bytes(),
        *b"a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0"
    );
    assert_eq!(
        format!("{:?}", a3a2a1a0),
        "a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0"
    );

    let g = a3a2a1a0;
    let gf = g.to_fields();
    let g_lowercase = g.to_utf8_bytes();

    // Verify interoperability between Guid and windows::core::GUID.
    {
        let v = GUID::from_u128(g.to_u128());
        assert_eq!(
            format!("{:?}", v).to_ascii_lowercase().as_bytes(),
            &g_lowercase
        );
        assert_eq!(v.to_u128(), a3a2a1a0.to_u128());
        assert_eq!(v, GUID::from_values(gf.0, gf.1, gf.2, gf.3));
        assert_eq!(v, GUID::from_u128(g.to_u128()));
        assert_eq!(g, Guid::from_fields(v.data1, v.data2, v.data3, v.data4));
        assert_eq!(g, Guid::from_u128(&v.to_u128()));
    }

    // Verify interoperability between Guid and uuid::Uuid.
    {
        let v = Uuid::from_u128(g.to_u128());
        let vf = v.as_fields();
        assert_eq!(
            format!("{:?}", v).to_ascii_lowercase().as_bytes(),
            &g_lowercase
        );

        assert_eq!(*v.as_bytes(), g.to_bytes_be());
        assert_eq!(v.to_bytes_le(), g.to_bytes_le());
        assert_eq!(v.as_fields(), (gf.0, gf.1, gf.2, &gf.3));
        assert_eq!(v.as_u128(), g.to_u128());

        assert_eq!(v, Uuid::from_bytes(g.to_bytes_be()));
        assert_eq!(v, Uuid::from_bytes_le(g.to_bytes_le()));
        assert_eq!(v, Uuid::from_fields(gf.0, gf.1, gf.2, &gf.3));
        assert_eq!(v, Uuid::from_u128(g.to_u128()));

        assert_eq!(g, Guid::from_bytes_be(v.as_bytes()));
        assert_eq!(g, Guid::from_bytes_le(&v.to_bytes_le()));
        assert_eq!(g, Guid::from_fields(vf.0, vf.1, vf.2, *vf.3));
        assert_eq!(g, Guid::from_u128(&v.as_u128()));
    }
}

#[cfg(all(windows))]
#[test]
fn guid_new() {
    use tlg::Guid;
    assert_ne!(Guid::new(), Guid::zero());
}

#[test]
fn meta_as_bytes() {
    let x = 47i32;
    let b = tli::meta_as_bytes(&x);
    assert_eq!(4, b.len());
    assert_eq!(&x as *const i32 as *const u8, b.as_ptr());
}

#[test]
fn tag_size() {
    assert_eq!(1, tli::tag_size(0));
    assert_eq!(1, tli::tag_size(0x0FE00000));
    assert_eq!(2, tli::tag_size(0x0FF00000));
    assert_eq!(4, tli::tag_size(0x0FFFFFFF));
}

#[test]
fn tag_encode() {
    assert_eq!([0x00], tli::tag_encode(0));
    assert_eq!([0x7F], tli::tag_encode(0x0FE00000));
    assert_eq!([0xFF, 0x40], tli::tag_encode(0x0FF00000));
    assert_eq!([0xFF, 0xFF, 0xFF, 0x7F], tli::tag_encode(0x0FFFFFFF));
}

#[test]
fn filetime_from_systemtime() {
    let epoch = std::time::SystemTime::UNIX_EPOCH;
    let d100 = std::time::Duration::from_secs(100);
    let after = epoch + d100;
    let before = epoch - d100;
    assert_eq!(
        tlg::filetime_from_systemtime!(after),
        tli::filetime_from_duration_after_1970(d100)
    );
    assert_eq!(
        tlg::filetime_from_systemtime!(before),
        tli::filetime_from_duration_before_1970(d100)
    );
}

#[test]
fn filetime_from_duration_after_1970() {
    use core::time::Duration;
    use tli::filetime_from_duration_after_1970 as ft_from_dur_after;
    const FT_PER_SEC: i64 = 10000000;
    let ft_1970 = 0x19DB1DED53E8000;
    let ft_30828_2 = 0x7FFF35F4F06C8000 - FT_PER_SEC * 2;
    let n000 = Duration::ZERO;
    let n001 = Duration::from_nanos(1);
    let n099 = Duration::from_nanos(99);
    let pos_max_2 = Duration::from_secs(0xD408314C00 - 2); // time between 1970 and 30828-2s

    // Adding to 1970

    // Test each second starting 2 seconds before boundary to 2 seconds after.
    for seconds in 0..4 {
        let dur_s = Duration::from_secs(seconds);
        let ft_s = (10000000 * seconds) as i64;

        // Test each tick starting 2 ticks before second to 2 ticks after.
        for ticks in 0..4 {
            let dur_ticks = Duration::from_nanos(ticks * 100);
            let ft_ticks = ticks as i64;

            assert_eq!(
                ft_clamp(ft_30828_2 + ft_s + ft_ticks),
                ft_from_dur_after(pos_max_2 + dur_s + dur_ticks + n099)
            );
            assert_eq!(
                ft_clamp(ft_30828_2 + ft_s + ft_ticks),
                ft_from_dur_after(pos_max_2 + dur_s + dur_ticks + n001)
            );
            assert_eq!(
                ft_clamp(ft_30828_2 + ft_s + ft_ticks),
                ft_from_dur_after(pos_max_2 + dur_s + dur_ticks + n000)
            );

            assert_eq!(
                ft_clamp(ft_1970 + ft_s + ft_ticks),
                ft_from_dur_after(dur_s + dur_ticks + n099)
            );
            assert_eq!(
                ft_clamp(ft_1970 + ft_s + ft_ticks),
                ft_from_dur_after(dur_s + dur_ticks + n001)
            );
            assert_eq!(
                ft_clamp(ft_1970 + ft_s + ft_ticks),
                ft_from_dur_after(dur_s + dur_ticks + n000)
            );
        }
    }
}

#[test]
fn filetime_from_duration_before_1970() {
    use core::time::Duration;
    use tli::filetime_from_duration_before_1970 as ft_from_dur_before;
    const FT_PER_SEC: i64 = 10000000;
    let ft_1601_2 = FT_PER_SEC * 2;
    let ft_1970 = 0x19DB1DED53E8000;
    let n000 = Duration::ZERO;
    let n001 = Duration::from_nanos(1);
    let n099 = Duration::from_nanos(99);
    let neg_max_2 = Duration::from_secs(0x2B6109100 - 2); // time between 1601+2s and 1970

    // Subtracting from 1970

    // Test each second starting 2 seconds before boundary to 2 seconds after.
    for seconds in 0..4 {
        let dur_s = Duration::from_secs(seconds);
        let ft_s = (10000000 * seconds) as i64;

        // Test each tick starting 2 ticks before second to 2 ticks after.
        for ticks in 0..4 {
            let dur_ticks = Duration::from_nanos(ticks * 100);
            let ft_ticks = ticks as i64;

            // Test rounding down at T-0.99, T-0.01, and T-0.00.
            assert_eq!(
                ft_clamp(ft_1601_2 - ft_s - ft_ticks - 1),
                ft_from_dur_before(neg_max_2 + dur_s + dur_ticks + n099)
            );
            assert_eq!(
                ft_clamp(ft_1601_2 - ft_s - ft_ticks - 1),
                ft_from_dur_before(neg_max_2 + dur_s + dur_ticks + n001)
            );
            assert_eq!(
                ft_clamp(ft_1601_2 - ft_s - ft_ticks),
                ft_from_dur_before(neg_max_2 + dur_s + dur_ticks + n000)
            );

            assert_eq!(
                ft_clamp(ft_1970 - ft_s - ft_ticks - 1),
                ft_from_dur_before(dur_s + dur_ticks + n099)
            );
            assert_eq!(
                ft_clamp(ft_1970 - ft_s - ft_ticks - 1),
                ft_from_dur_before(dur_s + dur_ticks + n001)
            );
            assert_eq!(
                ft_clamp(ft_1970 - ft_s - ft_ticks),
                ft_from_dur_before(dur_s + dur_ticks + n000)
            );
        }
    }
}

fn ft_clamp(ft: i64) -> i64 {
    let ft_1601 = 0;
    let ft_30828 = 0x7FFF35F4F06C8000;
    return if ft < ft_1601 {
        0
    } else if ft >= ft_30828 {
        ft_30828 - 1
    } else {
        ft
    };
}

struct Unregister(&'static tlg::Provider);

impl Drop for Unregister {
    fn drop(&mut self) {
        self.0.unregister();
    }
}

#[test]
fn define_provider() {
    tlg::define_provider!(PROV, "TraceLoggingDynamicTest");

    fn callback(
        _s: &tlg::Guid,
        _c: u32,
        _l: tlg::Level,
        _anyk: u64,
        _allk: u64,
        _flt: usize,
        ctx: usize,
    ) {
        assert_eq!(12345usize, ctx);
    }

    let _u = Unregister(&PROV);
    assert!(!PROV.enabled(tlg::Level::LogAlways, 0));
    PROV.unregister();
    assert!(!PROV.enabled(tlg::Level::LogAlways, 0));
    unsafe { PROV.register() };
    PROV.unregister();
    assert!(!PROV.enabled(tlg::Level::LogAlways, 0));
    PROV.unregister();
    assert!(!PROV.enabled(tlg::Level::LogAlways, 0));
    unsafe { PROV.register_with_callback(callback, 12345usize) };
    PROV.unregister();
    assert!(!PROV.enabled(tlg::Level::LogAlways, 0));
    PROV.unregister();
    assert!(!PROV.enabled(tlg::Level::LogAlways, 0));
    PROV.raw_meta();

    tlg::define_provider!(PROV1, "TestProvider1");
    assert_eq!("TestProvider1", PROV1.name());
    assert_eq!(&tlg::Guid::from_name("TestProvider1"), PROV1.id());

    tlg::define_provider!(
        PROV2,
        "TestProvider2",
        group_id("e3c21ff5-164e-4cbb-b08c-d47e2dcf9197")
    );
    assert_eq!("TestProvider2", PROV2.name());
    assert_eq!(&tlg::Guid::from_name("TestProvider2"), PROV2.id());

    tlg::define_provider!(
        PROV3,
        "TestProvider3",
        id("d93270c9-8144-4209-99a6-3acd39aaae71")
    );
    assert_eq!("TestProvider3", PROV3.name());
    assert_eq!(
        &tlg::Guid::from_u128(&0xd93270c9_8144_4209_99a6_3acd39aaae71),
        PROV3.id()
    );

    tlg::define_provider!(
        PROV4,
        "TestProvider4",
        id("632a8743-6a0d-456f-9ae8-a26febe2dbc3"),
        group_id("e3c21ff5-164e-4cbb-b08c-d47e2dcf9197")
    );
    assert_eq!("TestProvider4", PROV4.name());
    assert_eq!(
        &tlg::Guid::from_u128(&0x632a8743_6a0d_456f_9ae8_a26febe2dbc3),
        PROV4.id()
    );
}

#[test]
#[should_panic]
fn provider_panic() {
    tlg::define_provider!(PROV, "TraceLoggingDynamicTest");
    let _u = Unregister(&PROV);
    unsafe { PROV.register() };
    unsafe { PROV.register() };
}

#[test]
fn write_event() {
    tlg::define_provider!(PROV, "TraceLoggingDynamicTest");

    let _u = Unregister(&PROV);
    unsafe { PROV.register() };

    let sample_guid = tlg::Provider::guid_from_name("sample");
    let sample_filetime = 0x01d7ace794497cb5i64;
    let sample_rusttime =
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1671930123);
    let sample_systemtime = [2022, 1, 1, 2, 3, 4, 5, 6];
    let sample_sid = [1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0];
    let sample_ipv4 = [127, 0, 0, 1];
    let sample_ipv6 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    tlg::write_event!(PROV, "Default");

    tlg::write_event!(
        PROV,
        "4v2o6t123c0l3k11",
        id_version(4, 2),
        opcode(6),
        task(123),
        channel(0),
        level(3),
        keyword(0x11),
    );

    tlg::write_event!(PROV, "tag0xFE00000", tag(0xFE00000));
    tlg::write_event!(PROV, "tag0xFEDC000", tag(0xFEDC000));
    tlg::write_event!(PROV, "tag0xFEDCBAF", tag(0xFEDCBAF));

    tlg::write_event!(
        PROV,
        "fieldtag",
        u8("0xFE00000", &0, tag(0xFE00000)),
        u8("0xFEDC000", &0, tag(0xFEDC000)),
        u8("0xFEDCBAF", &0, tag(0xFEDCBAF)),
    );

    tlg::write_event!(
        PROV,
        "outtypes",
        u8("0xFE00000", &0, tag(0xFE00000)),
        u8("0xFEDC000", &0, tag(0xFEDC000)),
        u8("0xFEDCBAF", &0, tag(0xFEDCBAF)),
        u8("default100", &100),
        u8("string65", &65, format(String)),
        u8("bool1", &1, format(Boolean)),
        u8("hex100", &100, format(Hex)),
    );

    tlg::write_event!(PROV, "structs",
        u8("start", &0),
        struct("struct1", tag(0xFE00000), {
            u8("nested1", &1),
            struct("struct2", {
                u8("nested2", &2),
            }),
        }),
    );

    tlg::write_event!(
        PROV,
        "cstrs-L4-kFF",
        level(tlg::Level::Informational),
        keyword(0xFF),
        char8_cp1252("A", &b'A'),
        cstr16("cstr16-", &[]),
        cstr16("cstr16-a", &[97]),
        cstr16("cstr16-0", &[0]),
        cstr16("cstr16-a0", &[97, 0]),
        cstr16("cstr16-0a", &[0, 97]),
        cstr16("cstr16-a0a", &[97, 0, 97]),
        cstr8("cstr8-", ""),
        cstr8("cstr8-a", "a"),
        cstr8("cstr8-0", "\0"),
        cstr8("cstr8-a0", "a\0"),
        cstr8("cstr8-0a", "\0a"),
        cstr8("cstr8-a0a", "a\0a"),
        cstr8_cp1252("cstr8e-", ""),
        cstr8_cp1252("cstr8e-a", "a"),
        cstr8_cp1252("cstr8e-0", "\0"),
        cstr8_cp1252("cstr8e-a0", "a\0"),
        cstr8_cp1252("cstr8e-0a", "\0a"),
        cstr8_cp1252("cstr8e-a0a", "a\0a"),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "UnicodeString",
        char8_cp1252("A", &b'A'),
        cstr16("scalar", &Vec::from_iter("cstr-utf16".encode_utf16())),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "AnsiString",
        char8_cp1252("A", &b'A'),
        cstr8("scalar", "cstr-utf8"),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Int8",
        char8_cp1252("A", &b'A'),
        i8("scalar", &-8),
        i8_slice("a0", &[]),
        i8_slice("a1", &[-8]),
        i8_slice("a2", &[-8, -8]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "UInt8",
        char8_cp1252("A", &b'A'),
        u8("scalar", &8),
        u8_slice("a0", &[]),
        u8_slice("a1", &[8]),
        u8_slice("a2", &[8, 8]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Int16",
        char8_cp1252("A", &b'A'),
        i16("scalar", &-16),
        i16_slice("a0", &[]),
        i16_slice("a1", &[-16]),
        i16_slice("a2", &[-16, -16]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "UInt16",
        char8_cp1252("A", &b'A'),
        u16("scalar", &16),
        u16_slice("a0", &[]),
        u16_slice("a1", &[16]),
        u16_slice("a2", &[16, 16]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Int32",
        char8_cp1252("A", &b'A'),
        i32("scalar", &-32),
        i32_slice("a0", &[]),
        i32_slice("a1", &[-32]),
        i32_slice("a2", &[-32, -32]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "UInt32",
        char8_cp1252("A", &b'A'),
        u32("scalar", &32),
        u32_slice("a0", &[]),
        u32_slice("a1", &[32]),
        u32_slice("a2", &[32, 32]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Int64",
        char8_cp1252("A", &b'A'),
        i64("scalar", &-64),
        i64_slice("a0", &[]),
        i64_slice("a1", &[-64]),
        i64_slice("a2", &[-64, -64]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "UInt64",
        char8_cp1252("A", &b'A'),
        u64("scalar", &64),
        u64_slice("a0", &[]),
        u64_slice("a1", &[64]),
        u64_slice("a2", &[64, 64]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "IntPtr",
        char8_cp1252("A", &b'A'),
        isize("scalar", &-3264),
        isize_slice("a0", &[]),
        isize_slice("a1", &[-3264]),
        isize_slice("a2", &[-3264, -3264]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "UIntPtr",
        char8_cp1252("A", &b'A'),
        usize("scalar", &3264),
        usize_slice("a0", &[]),
        usize_slice("a1", &[3264]),
        usize_slice("a2", &[3264, 3264]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Float32",
        char8_cp1252("A", &b'A'),
        f32("scalar", &3.2),
        f32_slice("a0", &[]),
        f32_slice("a1", &[3.2]),
        f32_slice("a2", &[3.2, 3.2]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Float64",
        char8_cp1252("A", &b'A'),
        f64("scalar", &6.4),
        f64_slice("a0", &[]),
        f64_slice("a1", &[6.4]),
        f64_slice("a2", &[6.4, 6.4]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Bool8",
        char8_cp1252("A", &b'A'),
        bool8("scalar", &false),
        bool8_slice("a0", &[]),
        bool8_slice("a1", &[false]),
        bool8_slice("a2", &[false, false]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Bool8",
        char8_cp1252("A", &b'A'),
        bool8("scalar", &true),
        bool8_slice("a0", &[]),
        bool8_slice("a1", &[true]),
        bool8_slice("a2", &[true, true]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Bool32",
        char8_cp1252("A", &b'A'),
        bool32("scalar", &0),
        bool32_slice("a0", &[]),
        bool32_slice("a1", &[0]),
        bool32_slice("a2", &[0, 0]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Bool32",
        char8_cp1252("A", &b'A'),
        bool32("scalar", &1),
        bool32_slice("a0", &[]),
        bool32_slice("a1", &[1]),
        bool32_slice("a2", &[1, 1]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Binary",
        char8_cp1252("A", &b'A'),
        binary("scalar", "0123"),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Guid",
        char8_cp1252("A", &b'A'),
        guid("scalar", &sample_guid),
        guid_slice("a0", &[]),
        guid_slice("a1", &[sample_guid]),
        guid_slice("a2", &[sample_guid, sample_guid]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "FileTime",
        char8_cp1252("A", &b'A'),
        win_filetime("scalar", &sample_filetime),
        win_filetime_slice("a0", &[]),
        win_filetime_slice("a1", &[sample_filetime]),
        win_filetime_slice("a2", &[sample_filetime, sample_filetime]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "FileTime",
        char8_cp1252("A", &b'A'),
        systemtime("scalar", &sample_rusttime),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "SystemTime",
        char8_cp1252("A", &b'A'),
        win_systemtime("scalar", &sample_systemtime),
        win_systemtime_slice("a0", &[]),
        win_systemtime_slice("a1", &[sample_systemtime]),
        win_systemtime_slice("a2", &[sample_systemtime, sample_systemtime]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "Sid",
        char8_cp1252("A", &b'A'),
        win_sid("scalar", &sample_sid),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "HexInt32",
        char8_cp1252("A", &b'A'),
        i32_hex("scalar", &-559038737),
        i32_hex_slice("a0", &[]),
        i32_hex_slice("a1", &[-559038737]),
        i32_hex_slice("a2", &[-559038737, -559038737]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "HexInt32",
        char8_cp1252("A", &b'A'),
        u32_hex("scalar", &0xdeadbeef),
        u32_hex_slice("a0", &[]),
        u32_hex_slice("a1", &[0xdeadbeef]),
        u32_hex_slice("a2", &[0xdeadbeef, 0xdeadbeef]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "HexInt64",
        char8_cp1252("A", &b'A'),
        i64_hex("scalar", &-2401053088335073280),
        i64_hex_slice("a0", &[]),
        i64_hex_slice("a1", &[-2401053088335073280]),
        i64_hex_slice("a2", &[-2401053088335073280, -2401053088335073280]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "HexInt64",
        char8_cp1252("A", &b'A'),
        u64_hex("scalar", &0xdeadbeeffeeef000),
        u64_hex_slice("a0", &[]),
        u64_hex_slice("a1", &[0xdeadbeeffeeef000]),
        u64_hex_slice("a2", &[0xdeadbeeffeeef000, 0xdeadbeeffeeef000]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "HexIntPtr",
        char8_cp1252("A", &b'A'),
        isize_hex("scalar", &0x1234),
        isize_hex_slice("a0", &[]),
        isize_hex_slice("a1", &[0x1234]),
        isize_hex_slice("a2", &[0x1234, 0x1234]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "HexIntPtr",
        char8_cp1252("A", &b'A'),
        usize_hex("scalar", &0x1234),
        usize_hex_slice("a0", &[]),
        usize_hex_slice("a1", &[0x1234]),
        usize_hex_slice("a2", &[0x1234, 0x1234]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "CountedString",
        char8_cp1252("A", &b'A'),
        str16("scalar", &Vec::from_iter("utf16".encode_utf16())),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "CountedAnsiString",
        char8_cp1252("A", &b'A'),
        str8("scalar", "utf8"),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "BinaryC",
        char8_cp1252("A", &b'A'),
        binaryc("scalar", "0123"),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "IPv4",
        char8_cp1252("A", &b'A'),
        ipv4("scalar", &sample_ipv4),
        ipv4_slice("a0", &[]),
        ipv4_slice("a1", &[sample_ipv4]),
        ipv4_slice("a2", &[sample_ipv4, sample_ipv4]),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "IPv6",
        char8_cp1252("A", &b'A'),
        ipv6("scalar", &sample_ipv6),
        char8_cp1252("A", &b'A'),
    );

    tlg::write_event!(
        PROV,
        "IPv6c",
        char8_cp1252("A", &b'A'),
        ipv6c("scalar", &sample_ipv6),
        char8_cp1252("A", &b'A'),
    );
}
