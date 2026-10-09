use super::*;

#[test]
fn quantities_packaging_rounding_and_overflow() {
    let mut parcel = Parcel::default();
    parcel
        .add_item(Some(250), Some(131), Some(85), Some(21), 3)
        .unwrap();
    parcel
        .add_item(Some(400), Some(120), Some(100), Some(40), 2)
        .unwrap();
    let parcel = parcel
        .packaged(&Parcel {
            weight_g: 50,
            length_mm: 9,
            width_mm: 5,
            height_mm: 10,
        })
        .unwrap();
    assert_eq!(
        parcel,
        Parcel {
            weight_g: 1600,
            length_mm: 140,
            width_mm: 105,
            height_mm: 153
        }
    );
    assert_eq!(Parcel::centimeters(153).unwrap(), 16);
    assert_eq!(Parcel::centimeters(140).unwrap(), 14);
    assert_eq!(Parcel::centimeters(i64::MAX).unwrap(), i64::MAX / 10 + 1);
    assert!(
        Parcel::default()
            .add_item(None, Some(10), Some(10), Some(10), 1)
            .is_err()
    );
    assert!(
        Parcel::default()
            .add_item(Some(0), Some(10), Some(10), Some(10), 1)
            .is_err()
    );
    assert!(
        Parcel::default()
            .add_item(Some(1), Some(10), Some(10), Some(10), 0)
            .is_err()
    );
    assert!(
        Parcel {
            weight_g: i64::MAX,
            ..Default::default()
        }
        .add_item(Some(1), Some(10), Some(10), Some(10), 1)
        .is_err()
    );
    assert!(
        parcel
            .packaged(&Parcel {
                length_mm: -1,
                ..Default::default()
            })
            .is_err()
    );
}

#[test]
fn cep_validation() {
    assert_eq!(normalize_cep(" 01001-000 ").unwrap(), "01001000");
    for invalid in [
        "123",
        "123456789",
        "abcdefgh",
        "1234a5678",
        "１２３４５６７８",
    ] {
        assert!(normalize_cep(invalid).is_err());
    }
}

#[test]
fn options_are_deterministic() {
    let option = |code: &str, price, days| ShippingOption {
        id: code.into(),
        provider: "test".into(),
        service_code: code.into(),
        service_name: code.into(),
        price_cents: price,
        transit_days: Some(days),
        parcels: Vec::new(),
    };
    let mut options = vec![
        option("C", 100, 3),
        option("B", 100, 2),
        option("A", 100, 2),
        option("D", 90, 10),
    ];
    sort_options(&mut options);
    assert_eq!(
        options.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(),
        ["D", "A", "B", "C"]
    );
}
