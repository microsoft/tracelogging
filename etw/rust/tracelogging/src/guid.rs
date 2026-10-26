// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use core::borrow;
use core::convert::TryInto;
use core::fmt;
use core::mem;
use core::str::from_utf8;

/// [GUID](https://docs.microsoft.com/windows/win32/api/guiddef/ns-guiddef-guid)
/// ([UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier)).
/// with host-endian in-memory representation (as expected by the ETW APIs).
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

impl Guid {
    /// Returns a zeroed GUID, i.e. GUID_NULL.
    pub const fn zero() -> Self {
        return Self {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0; 8],
        };
    }

    /// Generates a unique GUID using UuidCreate.
    /// Note: For a zeroed GUID, use Guid::zero().
    /// ```
    /// # use tracelogging::Guid;
    /// let g = Guid::new();
    /// ```
    #[cfg(all(windows, not(proc_macro)))]
    pub fn new() -> Self {
        #[link(name = "rpcrt4")]
        extern "system" {
            fn UuidCreate(uuid: &mut Guid) -> u32;
        }

        let mut g = Guid::zero();

        // Safety: We assume that UuidCreate is well-behaved.
        unsafe {
            UuidCreate(&mut g);
        }
        return g;
    }

    /// Returns a GUID generated using Windows invariant casing and a hash of the specified trace
    /// provider name. The hash uses the same algorithm as many other ETW tools and APIs.
    /// Given the same name, it will always generate the same GUID.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///    Guid::from_name("MyProvider"),
    ///    Guid::from_u128(&0xb3864c38_4273_58c5_545b_8b3608343471));
    /// ```
    pub fn from_name(event_provider_name: &str) -> Self {
        let mut hasher = Sha1NonSecret::new();
        hasher.write(&[
            0x48, 0x2C, 0x2D, 0xB2, 0xC3, 0x90, 0x47, 0xC8, 0x87, 0xF8, 0x1A, 0x15, 0xBF, 0xC1,
            0x30, 0xFB,
        ]);

        // Hash name as uppercase UTF-16BE
        let mut u16buf = [0u16; 2];
        for upper_ch in event_provider_name.chars().map(uppercase_invariant) {
            for upper_u16 in upper_ch.encode_utf16(&mut u16buf) {
                hasher.write(&upper_u16.to_be_bytes());
            }
        }

        let mut v = hasher.finish();
        v[7] = (v[7] & 0x0F) | 0x50;
        return Guid::from_bytes_le(v[0..16].try_into().unwrap());
    }

    /// Creates a GUID from field values.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::from_bytes_be(&[0xa3, 0xa2, 0xa1, 0xa0, 0xb1, 0xb0, 0xc1, 0xc0, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]));
    /// ```
    pub const fn from_fields(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        return Self {
            data1,
            data2,
            data3,
            data4,
        };
    }

    /// Creates a GUID from bytes in big-endian (RFC) byte order.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::from_bytes_be(&[0xa3, 0xa2, 0xa1, 0xa0, 0xb1, 0xb0, 0xc1, 0xc0, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]));
    /// ```
    pub const fn from_bytes_be(bytes_be: &[u8; 16]) -> Self {
        let b = bytes_be;
        return Self {
            data1: u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
            data2: u16::from_be_bytes([b[4], b[5]]),
            data3: u16::from_be_bytes([b[6], b[7]]),
            data4: [b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]],
        };
    }

    /// Creates a GUID from bytes in little-endian byte order.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::from_bytes_le(&[0xa0, 0xa1, 0xa2, 0xa3, 0xb0, 0xb1, 0xc0, 0xc1, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]));
    /// ```
    pub const fn from_bytes_le(bytes_le: &[u8; 16]) -> Self {
        let b = bytes_le;
        return Self {
            data1: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            data2: u16::from_le_bytes([b[4], b[5]]),
            data3: u16::from_le_bytes([b[6], b[7]]),
            data4: [b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]],
        };
    }

    /// Creates a GUID from a u128 value.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::from_u128(&0xa3a2a1a0_b1b0_c1c0_d7d6d5d4d3d2d1d0));
    /// ```
    pub const fn from_u128(value: &u128) -> Self {
        return Self {
            data1: u32::from_le(value.wrapping_shr(96) as u32),
            data2: u16::from_le(value.wrapping_shr(80) as u16),
            data3: u16::from_le(value.wrapping_shr(64) as u16),
            data4: (*value as u64).to_be_bytes(),
        };
    }

    /// Creates a GUID from a string with optional {} and optional '-'.
    /// Returns None if GUID could not be parsed from the input.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::try_parse("{a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0}").unwrap());
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::try_parse("a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0").unwrap());
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::try_parse("a3a2a1a0b1b0c1c0d7d6d5d4d3d2d1d0").unwrap());
    /// ```
    pub fn try_parse(value: &str) -> Option<Self> {
        return Self::try_parse_ascii(value.as_bytes());
    }

    /// Creates a GUID from a string with optional {} and optional '-'.
    /// Returns None if GUID could not be parsed from the input.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::try_parse_ascii(b"{a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0}").unwrap());
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::try_parse_ascii(b"a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0").unwrap());
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]),
    ///     Guid::try_parse_ascii(b"a3a2a1a0b1b0c1c0d7d6d5d4d3d2d1d0").unwrap());
    /// ```
    pub fn try_parse_ascii(value: &[u8]) -> Option<Self> {
        if value.len() < 32 {
            return None;
        }

        let mut state = GuidParseState {
            input: if value[0] != b'{' || value[value.len() - 1] != b'}' {
                value
            } else {
                &value[1..value.len() - 1]
            },
            pos: 0,
            dash: 0,
            highbits: 0,
        };

        if (state.input.len() == 36)
            & (state.input[8] == b'-')
            & (state.input[13] == b'-')
            & (state.input[18] == b'-')
            & (state.input[23] == b'-')
        {
            state.dash = 1;
        } else if state.input.len() != 32 {
            return None;
        }

        let b = [
            state.next(),
            state.next(),
            state.next(),
            state.next(),
            state.next_dash(),
            state.next(),
            state.next_dash(),
            state.next(),
            state.next_dash(),
            state.next(),
            state.next_dash(),
            state.next(),
            state.next(),
            state.next(),
            state.next(),
            state.next(),
        ];

        if (state.highbits & 0xFFFFFF00) != 0 {
            return None;
        }

        return Some(Self::from_bytes_be(&b));
    }

    /// Returns the field values of the GUID as a tuple.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]).to_fields(),
    ///     (0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]));
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub const fn to_fields(&self) -> (u32, u16, u16, [u8; 8]) {
        return (self.data1, self.data2, self.data3, self.data4);
    }

    /// Returns this implementation's in-memory byte representation.
    pub const fn as_bytes_raw(&self) -> &[u8; 16] {
        return unsafe { mem::transmute::<&Guid, &[u8; 16]>(self) };
    }

    /// Returns the bytes of the GUID in big-endian (RFC) byte order.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]).to_bytes_be(),
    ///     [0xa3, 0xa2, 0xa1, 0xa0, 0xb1, 0xb0, 0xc1, 0xc0, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]);
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub const fn to_bytes_be(&self) -> [u8; 16] {
        return [
            (self.data1 >> 24) as u8,
            (self.data1 >> 16) as u8,
            (self.data1 >> 8) as u8,
            self.data1 as u8,
            (self.data2 >> 8) as u8,
            self.data2 as u8,
            (self.data3 >> 8) as u8,
            self.data3 as u8,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7],
        ];
    }

    /// Returns the bytes of the GUID in little-endian byte order.
    /// ```
    /// # use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]).to_bytes_le(),
    ///     [0xa0, 0xa1, 0xa2, 0xa3, 0xb0, 0xb1, 0xc0, 0xc1, 0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]);
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub const fn to_bytes_le(&self) -> [u8; 16] {
        return [
            self.data1 as u8,
            (self.data1 >> 8) as u8,
            (self.data1 >> 16) as u8,
            (self.data1 >> 24) as u8,
            self.data2 as u8,
            (self.data2 >> 8) as u8,
            self.data3 as u8,
            (self.data3 >> 8) as u8,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7],
        ];
    }

    /// Returns the GUID as a u128 value.
    /// ```
    /// use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]).to_u128(),
    ///     0xa3a2a1a0_b1b0_c1c0_d7d6d5d4d3d2d1d0);
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub const fn to_u128(&self) -> u128 {
        return (self.data1.to_le() as u128) << 96
            | (self.data2.to_le() as u128) << 80
            | (self.data3.to_le() as u128) << 64
            | u64::from_be_bytes(self.data4) as u128;
    }

    /// Convert GUID to utf8 string bytes.
    /// To get a &str, use: `str::from_utf8(&guid.to_utf8_bytes()).unwrap()`.
    /// ```
    /// use tracelogging::Guid;
    /// assert_eq!(
    ///     Guid::from_fields(0xa3a2a1a0, 0xb1b0, 0xc1c0, [0xd7, 0xd6, 0xd5, 0xd4, 0xd3, 0xd2, 0xd1, 0xd0]).to_utf8_bytes(),
    ///     *b"a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0");
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub const fn to_utf8_bytes(&self) -> [u8; 36] {
        const HEX_DIGITS: &[u8] = b"0123456789abcdef";
        return [
            HEX_DIGITS[((self.data1 >> 28) & 0xf) as usize],
            HEX_DIGITS[((self.data1 >> 24) & 0xf) as usize],
            HEX_DIGITS[((self.data1 >> 20) & 0xf) as usize],
            HEX_DIGITS[((self.data1 >> 16) & 0xf) as usize],
            HEX_DIGITS[((self.data1 >> 12) & 0xf) as usize],
            HEX_DIGITS[((self.data1 >> 8) & 0xf) as usize],
            HEX_DIGITS[((self.data1 >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data1 & 0xf) as usize],
            b'-',
            HEX_DIGITS[((self.data2 >> 12) & 0xf) as usize],
            HEX_DIGITS[((self.data2 >> 8) & 0xf) as usize],
            HEX_DIGITS[((self.data2 >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data2 & 0xf) as usize],
            b'-',
            HEX_DIGITS[((self.data3 >> 12) & 0xf) as usize],
            HEX_DIGITS[((self.data3 >> 8) & 0xf) as usize],
            HEX_DIGITS[((self.data3 >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data3 & 0xf) as usize],
            b'-',
            HEX_DIGITS[((self.data4[0] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[0] & 0xf) as usize],
            HEX_DIGITS[((self.data4[1] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[1] & 0xf) as usize],
            b'-',
            HEX_DIGITS[((self.data4[2] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[2] & 0xf) as usize],
            HEX_DIGITS[((self.data4[3] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[3] & 0xf) as usize],
            HEX_DIGITS[((self.data4[4] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[4] & 0xf) as usize],
            HEX_DIGITS[((self.data4[5] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[5] & 0xf) as usize],
            HEX_DIGITS[((self.data4[6] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[6] & 0xf) as usize],
            HEX_DIGITS[((self.data4[7] >> 4) & 0xf) as usize],
            HEX_DIGITS[(self.data4[7] & 0xf) as usize],
        ];
    }
}

impl fmt::Debug for Guid {
    /// Format the GUID, e.g. "a3a2a1a0-b1b0-c1c0-d7d6-d5d4d3d2d1d0".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.write_str(from_utf8(&self.to_utf8_bytes()).unwrap());
    }
}

impl borrow::Borrow<[u8; 16]> for Guid {
    /// Returns this implementation's in-memory byte representation.
    fn borrow(&self) -> &[u8; 16] {
        return unsafe { mem::transmute::<&Guid, &[u8; 16]>(self) };
    }
}

fn uppercase_invariant(ch: char) -> char {
    // Native Windows invariant casing, shared with tracelogging_macros. Unlike
    // Unicode full uppercase, these mappings never expand a character.
    const RANGES: &[(u32, u32, u32, i32)] = &[
        (0x0061, 0x007a, 1, -32),
        (0x00e0, 0x00f6, 1, -32),
        (0x00f8, 0x00fe, 1, -32),
        (0x00ff, 0x00ff, 1, 121),
        (0x0101, 0x012f, 2, -1),
        (0x0133, 0x0137, 2, -1),
        (0x013a, 0x0148, 2, -1),
        (0x014b, 0x0177, 2, -1),
        (0x017a, 0x017e, 2, -1),
        (0x0180, 0x0180, 1, 195),
        (0x0183, 0x0185, 2, -1),
        (0x0188, 0x0188, 1, -1),
        (0x018c, 0x018c, 1, -1),
        (0x0192, 0x0192, 1, -1),
        (0x0195, 0x0195, 1, 97),
        (0x0199, 0x0199, 1, -1),
        (0x019a, 0x019a, 1, 163),
        (0x019e, 0x019e, 1, 130),
        (0x01a1, 0x01a5, 2, -1),
        (0x01a8, 0x01a8, 1, -1),
        (0x01ad, 0x01ad, 1, -1),
        (0x01b0, 0x01b0, 1, -1),
        (0x01b4, 0x01b6, 2, -1),
        (0x01b9, 0x01b9, 1, -1),
        (0x01bd, 0x01bd, 1, -1),
        (0x01bf, 0x01bf, 1, 56),
        (0x01c6, 0x01c6, 1, -2),
        (0x01c9, 0x01c9, 1, -2),
        (0x01cc, 0x01cc, 1, -2),
        (0x01ce, 0x01dc, 2, -1),
        (0x01dd, 0x01dd, 1, -79),
        (0x01df, 0x01ef, 2, -1),
        (0x01f3, 0x01f3, 1, -2),
        (0x01f5, 0x01f5, 1, -1),
        (0x01f9, 0x021f, 2, -1),
        (0x0223, 0x0233, 2, -1),
        (0x023c, 0x023c, 1, -1),
        (0x0242, 0x0242, 1, -1),
        (0x0247, 0x024f, 2, -1),
        (0x0250, 0x0250, 1, 10783),
        (0x0251, 0x0251, 1, 10780),
        (0x0253, 0x0253, 1, -210),
        (0x0254, 0x0254, 1, -206),
        (0x0256, 0x0257, 1, -205),
        (0x0259, 0x0259, 1, -202),
        (0x025b, 0x025b, 1, -203),
        (0x0260, 0x0260, 1, -205),
        (0x0263, 0x0263, 1, -207),
        (0x0268, 0x0268, 1, -209),
        (0x0269, 0x0269, 1, -211),
        (0x026b, 0x026b, 1, 10743),
        (0x026f, 0x026f, 1, -211),
        (0x0271, 0x0271, 1, 10749),
        (0x0272, 0x0272, 1, -213),
        (0x0275, 0x0275, 1, -214),
        (0x027d, 0x027d, 1, 10727),
        (0x0280, 0x0280, 1, -218),
        (0x0283, 0x0283, 1, -218),
        (0x0288, 0x0288, 1, -218),
        (0x0289, 0x0289, 1, -69),
        (0x028a, 0x028b, 1, -217),
        (0x028c, 0x028c, 1, -71),
        (0x0292, 0x0292, 1, -219),
        (0x0371, 0x0373, 2, -1),
        (0x0377, 0x0377, 1, -1),
        (0x037b, 0x037d, 1, 130),
        (0x03ac, 0x03ac, 1, -38),
        (0x03ad, 0x03af, 1, -37),
        (0x03b1, 0x03c1, 1, -32),
        (0x03c3, 0x03cb, 1, -32),
        (0x03cc, 0x03cc, 1, -64),
        (0x03cd, 0x03ce, 1, -63),
        (0x03d7, 0x03d7, 1, -8),
        (0x03d9, 0x03ef, 2, -1),
        (0x03f2, 0x03f2, 1, 7),
        (0x03f8, 0x03f8, 1, -1),
        (0x03fb, 0x03fb, 1, -1),
        (0x0430, 0x044f, 1, -32),
        (0x0450, 0x045f, 1, -80),
        (0x0461, 0x0481, 2, -1),
        (0x048b, 0x04bf, 2, -1),
        (0x04c2, 0x04ce, 2, -1),
        (0x04cf, 0x04cf, 1, -15),
        (0x04d1, 0x0523, 2, -1),
        (0x0561, 0x0586, 1, -48),
        (0x1d79, 0x1d79, 1, 35332),
        (0x1d7d, 0x1d7d, 1, 3814),
        (0x1e01, 0x1e95, 2, -1),
        (0x1ea1, 0x1eff, 2, -1),
        (0x1f00, 0x1f07, 1, 8),
        (0x1f10, 0x1f15, 1, 8),
        (0x1f20, 0x1f27, 1, 8),
        (0x1f30, 0x1f37, 1, 8),
        (0x1f40, 0x1f45, 1, 8),
        (0x1f51, 0x1f57, 2, 8),
        (0x1f60, 0x1f67, 1, 8),
        (0x1f70, 0x1f71, 1, 74),
        (0x1f72, 0x1f75, 1, 86),
        (0x1f76, 0x1f77, 1, 100),
        (0x1f78, 0x1f79, 1, 128),
        (0x1f7a, 0x1f7b, 1, 112),
        (0x1f7c, 0x1f7d, 1, 126),
        (0x1f80, 0x1f87, 1, 8),
        (0x1f90, 0x1f97, 1, 8),
        (0x1fa0, 0x1fa7, 1, 8),
        (0x1fb0, 0x1fb1, 1, 8),
        (0x1fb3, 0x1fb3, 1, 9),
        (0x1fc3, 0x1fc3, 1, 9),
        (0x1fd0, 0x1fd1, 1, 8),
        (0x1fe0, 0x1fe1, 1, 8),
        (0x1fe5, 0x1fe5, 1, 7),
        (0x1ff3, 0x1ff3, 1, 9),
        (0x214e, 0x214e, 1, -28),
        (0x2170, 0x217f, 1, -16),
        (0x2184, 0x2184, 1, -1),
        (0x24d0, 0x24e9, 1, -26),
        (0x2c30, 0x2c5e, 1, -48),
        (0x2c61, 0x2c61, 1, -1),
        (0x2c65, 0x2c65, 1, -10795),
        (0x2c66, 0x2c66, 1, -10792),
        (0x2c68, 0x2c6c, 2, -1),
        (0x2c73, 0x2c73, 1, -1),
        (0x2c76, 0x2c76, 1, -1),
        (0x2c81, 0x2ce3, 2, -1),
        (0x2d00, 0x2d25, 1, -7264),
        (0xa641, 0xa65f, 2, -1),
        (0xa663, 0xa66d, 2, -1),
        (0xa681, 0xa697, 2, -1),
        (0xa723, 0xa72f, 2, -1),
        (0xa733, 0xa76f, 2, -1),
        (0xa77a, 0xa77c, 2, -1),
        (0xa77f, 0xa787, 2, -1),
        (0xa78c, 0xa78c, 1, -1),
        (0xff41, 0xff5a, 1, -32),
        (0x10428, 0x1044f, 1, -40),
    ];

    let value = ch as u32;
    let mut first = 0;
    let mut last = RANGES.len();
    while first != last {
        let mid = first + (last - first) / 2;
        let (start, end, step, delta) = RANGES[mid];
        if value < start {
            last = mid;
        } else if value > end {
            first = mid + 1;
        } else {
            return if (value - start) % step == 0 {
                char::from_u32((value as i32 + delta) as u32).unwrap()
            } else {
                ch
            };
        }
    }
    return ch;
}

#[cfg(all(test, windows))]
#[test]
fn uppercase_invariant_matches_windows() {
    #[link(name = "kernel32")]
    extern "system" {
        fn LCMapStringEx(
            locale: *const u16,
            flags: u32,
            source: *const u16,
            source_len: i32,
            destination: *mut u16,
            destination_len: i32,
            version: usize,
            reserved: usize,
            sort_handle: isize,
        ) -> i32;
    }

    for value in 0..=0x10ffff {
        if let Some(ch) = char::from_u32(value) {
            let mut source = [0; 2];
            let source = ch.encode_utf16(&mut source);
            let mut destination = [0; 2];
            let length = unsafe {
                LCMapStringEx(
                    [0].as_ptr(),
                    0x200,
                    source.as_ptr(),
                    source.len() as i32,
                    destination.as_mut_ptr(),
                    destination.len() as i32,
                    0,
                    0,
                    0,
                )
            };
            assert_eq!(length as usize, source.len());
            let mut expected = [0; 2];
            let expected = uppercase_invariant(ch).encode_utf16(&mut expected);
            assert_eq!(expected, &destination[..length as usize], "U+{:04X}", value);
        }
    }
}

struct GuidParseState<'a> {
    input: &'a [u8],
    pos: usize,
    dash: usize,
    highbits: u32,
}

impl GuidParseState<'_> {
    fn next(&mut self) -> u8 {
        let val1 = Self::hex_to_u4(self.input[self.pos]);
        let val2 = Self::hex_to_u4(self.input[self.pos + 1]);
        self.pos += 2;
        let val = (val1 << 4) | val2;
        self.highbits |= val;
        return val as u8;
    }

    fn next_dash(&mut self) -> u8 {
        self.pos += self.dash;
        return self.next();
    }

    fn hex_to_u4(ch: u8) -> u32 {
        let mut index = ch.wrapping_sub(48);
        let hexval = if index < 10 {
            index as u32
        } else {
            index = (ch | 32).wrapping_sub(97);
            if index < 6 { (index + 10) as u32 } else { 256 }
        };
        return hexval;
    }
}

/// Single-use SHA1 hasher (finish() is destructive). Note that this implementation
/// is for hashing public information. Do not use this code to hash private data
/// as this implementation does not take any steps to avoid information disclosure
/// (i.e. does not scrub its buffers).
struct Sha1NonSecret {
    chunk: [u8; 64],  // Each chunk is 64 bytes.
    chunk_count: u32, // Implementation limited to 2^32-1 chunks = 255GB.
    chunk_pos: u8,
    results: [u32; 5],
}

impl Sha1NonSecret {
    pub fn new() -> Sha1NonSecret {
        return Self {
            chunk: [0; 64],
            chunk_count: 0,
            chunk_pos: 0,
            results: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
        };
    }

    pub fn write_u8(&mut self, val: u8) {
        self.chunk[self.chunk_pos as usize] = val;
        self.chunk_pos = (self.chunk_pos + 1) & 63;
        if self.chunk_pos == 0 {
            self.drain();
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for i in bytes {
            self.write_u8(*i);
        }
    }

    pub fn finish(&mut self) -> [u8; 20] {
        // Need to capture chunk_count before we add end-bit and zerofill.
        let total_bit_count = (self.chunk_count as u64 * 512) + (self.chunk_pos as u64 * 8);

        // Add end-bit
        self.write_u8(0x80);

        // Zero-fill until almost to end of chunk.
        while self.chunk_pos != 56 {
            self.write_u8(0);
        }

        // End chunk with total bit count.
        self.write(&total_bit_count.to_be_bytes());
        debug_assert_eq!(self.chunk_pos, 0, "Bug: write should have drained");

        let mut sha1 = [0u8; 20];
        for i in 0..5 {
            sha1[(i * 4)..(i * 4 + 4)].copy_from_slice(&self.results[i].to_be_bytes());
        }

        return sha1;
    }

    fn drain(&mut self) {
        let mut w = [0u32; 80];

        let mut wpos = 0;
        while wpos != 16 {
            w[wpos] = u32::from_be_bytes([
                self.chunk[wpos * 4],
                self.chunk[wpos * 4 + 1],
                self.chunk[wpos * 4 + 2],
                self.chunk[wpos * 4 + 3],
            ]);
            wpos += 1;
        }

        while wpos != 80 {
            w[wpos] = (w[wpos - 3] ^ w[wpos - 8] ^ w[wpos - 14] ^ w[wpos - 16]).rotate_left(1);
            wpos += 1;
        }

        let mut a = self.results[0];
        let mut b = self.results[1];
        let mut c = self.results[2];
        let mut d = self.results[3];
        let mut e = self.results[4];

        wpos = 0;
        while wpos != 20 {
            const K: u32 = 0x5A827999;
            let f = (b & c) | (!b & d);
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(K)
                .wrapping_add(w[wpos]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
            wpos += 1;
        }

        while wpos != 40 {
            const K: u32 = 0x6ED9EBA1;
            let f = b ^ c ^ d;
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(K)
                .wrapping_add(w[wpos]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
            wpos += 1;
        }

        while wpos != 60 {
            const K: u32 = 0x8F1BBCDC;
            let f = (b & c) | (b & d) | (c & d);
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(K)
                .wrapping_add(w[wpos]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
            wpos += 1;
        }

        while wpos != 80 {
            const K: u32 = 0xCA62C1D6;
            let f = b ^ c ^ d;
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(K)
                .wrapping_add(w[wpos]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
            wpos += 1;
        }

        self.results[0] = self.results[0].wrapping_add(a);
        self.results[1] = self.results[1].wrapping_add(b);
        self.results[2] = self.results[2].wrapping_add(c);
        self.results[3] = self.results[3].wrapping_add(d);
        self.results[4] = self.results[4].wrapping_add(e);
        self.chunk_count += 1;
    }
}
