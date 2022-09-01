extern crate alloc;

#[test]
fn guid() {
    use crate::guid::Guid;
    use alloc::format;
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

#[cfg(all(windows, not(feature = "no_windows")))]
#[test]
fn guid_new() {
    use crate::guid::Guid;
    assert_ne!(Guid::new(), Guid::zero());
}
