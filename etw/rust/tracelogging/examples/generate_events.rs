// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use std::time::SystemTime;
use tracelogging as tlg;

tlg::define_provider!(PROV1, "TestProvider");

tlg::define_provider!(
    PROV2,
    "TestProvider2",
    id("97c801ee-c28b-5bb6-2ae4-11e18fe6137a"),
);

tlg::define_provider!(
    PROV3,
    "TestProvider3",
    group_id("12345678-9abc-def0-1234-56789abcdef0"),
);

tlg::define_provider!(
    PROV4,
    "TestProvider4",
    id("97c801ee-c28b-5bb6-2ae4-11e18fe6137a"),
    group_id("12345678-9abc-def0-1234-56789abcdef0"),
);

fn main() {
    unsafe {
        PROV1.register();
    }

    let guid1 = tlg::Guid::from_name("H1");
    let guid2 = tlg::Guid::from_name("H2");
    let string = String::from("Hello");

    // Normal and struct fields
    let _win32result = tlg::write_event!(

        // Required event attributes:

        PROV1,
        "MyEventName",

        // Optional event attributes:

        activity_id(&guid1), // activity_id and related_id can be either &Guid or &[u8; 16].
        related_id(guid2.as_bytes_raw()),
        channel(tlg::Channel::TraceLogging),
        level(Informational),
        opcode(0),
        task(6),
        keyword(0),     // If no keyword specified, default is keyword(1).
        keyword(0x10),  // If multiple keyword specified, they will be or'ed.
        tag(0x10000000 - 1),
        id_version(1, 0),

        // Fields:

        bool8("bool8_field", &true),
        binary("bin123", &[1, 2, 3]),
        guid_slice("guids!", &[guid1, guid2]),
        guid("guid_with_tag", &guid1, tag(0x10000000 - 1)),
        u8("u8_as_char", &65, tag(1), format(String)), // equivalent to char8_cp1252

        struct("MyStruct", tag(0x123), {
            u8("nested-u8", &1),
            struct("NestedStruct", {
                guid_slice("MoreGuids", &[guid1, guid2]),
            }),
        }),

        str8("str8", "counted utf-8"),          // str8 and cstr8 expect &[u8] containing utf-8
        cstr8("cstr8", "nul-terminated utf-8"), // cstr8 is nul-terminated in event, but input doesn't need to be nul-terminated.
        str8_cp1252("str8_cp1252", "counted cp1252"), // str8_cp1252 and cstr8_cp1252 expect &[u8] containing ANSI text.
        cstr8_cp1252("cstr8_cp1252", "nul-terminated cp1252"),
        str16("str16", &[65, 66, 67]),    // str16 and cstr16 expect &[u16] containing utf-16
        cstr16("cstr16", &[65, 66, 67]),

        // str8, cstr8, str16, cstr16 all come in json and xml flavors.
        str8_json("str8_json", "\"json\""),
        cstr16_xml("cstr16_xml", &[b'<' as u16, b'x' as u16, b'm' as u16, b'l' as u16, b'/' as u16, b'>' as u16]),

        str8("str8", "str8_val"),   // AsRef unwraps &str --> &[u8]
        str8("string", &string),    // AsRef unwraps &String --> &[u8]

        ipv4("ipv4", &[127, 0, 0, 1]),
        ipv6("ipv6", &[1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6,]),
        pointer("pointer", &1234),
        codepointer("codepointer", &(main as *const u8 as usize)),

        systemtime("now", &SystemTime::now()),
        hresult("E_FILENOTFOUND", &-2147024894),
        time32("t32_100", &100),
        time64("t64_200", &200),
        errno("errno", &2),
        errno_slice("errno_slice", &[1, 2]),

        win_ntstatus("STATUS_ACCESS_VIOLATION", &-1073741819),
        win_error("ERROR_ACCESS_DENIED", &5),
        win_sid("sid", &[1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]),
        win_filetime("ft", &0x19DB1DED53E8000), // 1970-01-01
        win_systemtime("st", &[2022, 8, 1, 22, 5, 6, 7, 8]),
        win_systemtime_utc("st_utc", &[2022, 8, 1, 22, 5, 6, 7, 8]),
    );

    // Raw fields
    let _win32_result3 = tlg::write_event!(
        PROV1,
        "Raw",
        //debug(),
        raw_field("RawChar8", U8, &[65], format(String), tag(200)),
        raw_field_slice("RawChar8s", U8, &[3, 0, 65, 66, 67], format(String)),
        raw_meta("RawHex32", U32, format(Hex)),
        raw_meta_slice("RawHex8s", U8, format(Hex)),
        raw_data(&[255, 0, 0, 0, 3, 0, 65, 66, 67]),
        raw_struct("RawStruct", 2),
        raw_field("RawChar8", U8, &[65], format(String)),
        raw_field_slice("RawChar8s", U8, &[3, 0, 65, 66, 67], format(String)),
        raw_struct_slice("RawStructSlice", 2),
        raw_meta("RawChar8", U8, format(String)),
        raw_meta_slice("RawChar8s", U8, format(String)),
        raw_data(&[
            2, 0, // 2 structs in the struct array
            48, 3, 0, 65, 66, 67, 49, 2, 0, 48, 49
        ]),
    );

    println!(
        "PROV1={:?}, L5K1={}, L4K10={}",
        PROV1,
        tlg::provider_enabled!(PROV1, tlg::Level::from_int(5), 1),
        tlg::provider_enabled!(PROV1, tlg::Level::Informational, 0x10),
    );
    PROV1.unregister();
}
