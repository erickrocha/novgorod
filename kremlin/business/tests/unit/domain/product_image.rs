    use super::*;

    #[test]
    fn test_compute_storage_identity_hash() {
        let hash = compute_storage_identity_hash("s3", "test-bucket", "products/1/photo.jpg");
        assert_eq!(hash.len(), 32);

        let hash2 = compute_storage_identity_hash("s3", "test-bucket", "products/1/photo.jpg");
        assert_eq!(hash, hash2);

        let hash_different = compute_storage_identity_hash("s3", "test-bucket", "products/2/photo.jpg");
        assert_ne!(hash, hash_different);
    }

    #[test]
    fn test_binary_uuid_conversion() {
        let original_uuid = Uuid::new_v4();
        let bytes = original_uuid.as_bytes().to_vec();
        let uuid_str = binary_to_uuid_string(&bytes);
        assert_eq!(uuid_str, original_uuid.to_string());

        let converted_bytes = uuid_string_to_binary(&uuid_str);
        assert_eq!(bytes, converted_bytes);
    }

    #[test]
    fn test_entity_mapper_from_model() {
        let uuid = Uuid::new_v4();
        let model = Model {
            id: 10,
            uuid: uuid.as_bytes().to_vec(),
            tenant_id: Some(1),
            product_id: 42,
            sku_id: None,
            alt_text: Some("Sample Image".to_string()),
            sort_order: 1,
            is_primary: true,
            storage_provider: "s3".to_string(),
            bucket: "novgorod-media-dev".to_string(),
            object_key: "tenants/1/products/42/images/sample.jpg".to_string(),
            storage_identity_hash: vec![1, 2, 3],
            object_version: None,
            etag: Some("\"etag123\"".to_string()),
            checksum_sha256: None,
            original_filename: "sample.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size_bytes: 2048,
            width_px: Some(800),
            height_px: Some(600),
            storage_status: STATUS_AVAILABLE.to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            created_by: None,
            updated_at: chrono::Utc::now().naive_utc(),
            updated_by: None,
            deleted_at: None,
            deleted_by: None,
        };

        let domain = ProductImageEntityMapper::from_model(model);
        assert_eq!(domain.id, Some(10));
        assert_eq!(domain.product_id, 42);
        assert_eq!(domain.uuid.unwrap(), uuid.to_string());
        assert_eq!(domain.is_primary, true);
        assert_eq!(domain.storage_status, STATUS_AVAILABLE);
        assert_eq!(domain.original_filename, "sample.jpg");
    }
