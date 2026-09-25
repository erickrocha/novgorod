    use super::*;
    fn input() -> CreatePurchaseInput {
        CreatePurchaseInput {
            items: vec![
                PurchaseItemInput {
                    sku_id: 2,
                    quantity: 1,
                },
                PurchaseItemInput {
                    sku_id: 1,
                    quantity: 3,
                },
                PurchaseItemInput {
                    sku_id: 2,
                    quantity: 2,
                },
            ],
            shipping_address: AddressInput {
                recipient: " Buyer ".into(),
                address_line1: "Street 1".into(),
                address_line2: None,
                locality: "City".into(),
                administrative_area: "SP".into(),
                postal_code: "01001000".into(),
                country_code: "br".into(),
            },
            billing_address: None,
        }
    }
    #[test]
    fn canonical_retries_and_address_copy() {
        let normalized = input().normalize().unwrap();
        assert_eq!(
            normalized.items,
            vec![
                PurchaseItemInput {
                    sku_id: 1,
                    quantity: 3
                },
                PurchaseItemInput {
                    sku_id: 2,
                    quantity: 3
                }
            ]
        );
        assert_eq!(
            normalized.billing_address.as_ref(),
            Some(&normalized.shipping_address)
        );
        let mut equivalent = normalized.clone();
        equivalent.items.reverse();
        assert_eq!(
            normalized.request_hash(),
            equivalent.normalize().unwrap().request_hash()
        );
    }
    #[test]
    fn reject_invalid_quantities_and_overflow() {
        let mut value = input();
        value.items[0].quantity = 0;
        assert!(value.normalize().is_err());
        let mut value = input();
        value.items[0].quantity = i32::MAX;
        assert!(value.normalize().is_err());
        let mut value = input();
        value.items.clear();
        assert!(value.normalize().is_err());
    }
    #[test]
    fn rejects_client_controlled_fields() {
        let mut value = serde_json::to_value(input()).unwrap();
        value["totalCents"] = 1.into();
        assert!(serde_json::from_value::<CreatePurchaseInput>(value).is_err());
    }
