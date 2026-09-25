    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_wine_catalog_csv_deserialization_50_wines() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let csv_path = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("data/wines/wine_catalog.csv");

        let content = std::fs::read(&csv_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", csv_path.display(), e));

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(content.as_slice());

        let rows: Vec<CatalogRow> = reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .expect("CSV deserialization of 50 wines should succeed without error");

        assert_eq!(rows.len(), 50, "Catalog CSV must contain exactly 50 wines");

        let mut unique_slugs = HashSet::new();
        let mut unique_codes = HashSet::new();
        let mut categories = HashSet::new();
        let mut countries = HashSet::new();

        for row in &rows {
            assert!(!row.product_name.trim().is_empty(), "Product name must not be empty");
            assert!(!row.slug.trim().is_empty(), "Slug must not be empty");
            assert!(!row.code.trim().is_empty(), "Code must not be empty");
            assert!(row.price_cents > 0, "Price cents must be positive for {}", row.product_name);
            assert_eq!(row.ncm, "22042100", "NCM must be 22042100 for {}", row.product_name);
            assert_eq!(row.cest.as_deref(), Some("0301200"), "CEST must be 0301200 for {}", row.product_name);
            assert!(row.brand.is_some(), "Brand must be present for {}", row.product_name);
            assert!(row.country.is_some(), "Country must be present for {}", row.product_name);
            assert!(row.region.is_some(), "Region must be present for {}", row.product_name);
            assert!(row.grape.is_some(), "Grape must be present for {}", row.product_name);
            assert!(row.vintage.is_some(), "Vintage must be present for {}", row.product_name);
            assert!(row.alcohol.is_some(), "Alcohol must be present for {}", row.product_name);
            assert!(row.stock_quantity.unwrap_or(0) > 0, "Stock must be positive for {}", row.product_name);
            assert!(row.active, "Product must be active for {}", row.product_name);

            if row.country.as_deref() == Some("Brasil") {
                assert_eq!(row.origem_mercadoria, 0, "Brazilian wines must have origem 0: {}", row.product_name);
            } else {
                assert_eq!(row.origem_mercadoria, 1, "Imported wines must have origem 1: {}", row.product_name);
            }

            unique_slugs.insert(row.slug.clone());
            unique_codes.insert(row.code.clone());
            categories.insert(row.category_key.clone());
            countries.insert(row.country.clone().unwrap());
        }

        assert_eq!(unique_slugs.len(), 50, "All 50 wine slugs must be unique");
        assert_eq!(unique_codes.len(), 50, "All 50 SKU codes must be unique");
        assert!(categories.len() >= 4, "Must cover at least 4 wine categories, found: {:?}", categories);
        assert!(countries.contains("Brasil"));
        assert!(countries.contains("Argentina"));
        assert!(countries.contains("Chile"));
        assert!(countries.contains("França"));
        assert!(countries.contains("Itália"));
        assert!(countries.contains("Portugal"));
        assert!(countries.contains("Espanha"));
    }

    #[test]
    fn test_lenient_csv_deserialization_with_empty_fields() {
        let csv_data = "product_key,category_key,category_name,product_name,slug,description,brand,ncm,cest,origem_mercadoria,code,variant_key,price_cents,compare_at_price_cents,stock_quantity,weight_g,width_mm,height_mm,length_mm,active,country,region,grape,vintage,alcohol,volume,color\n\
        pk-1,vinhos-tintos,Vinhos Tintos,Vinho Teste,vinho-teste,,,22042100,,0,SKU-001,750ml,9900,,,,,,,true,Brasil,,,,,,Tinto\n";

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let rows: Vec<CatalogRow> = reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .expect("Should deserialize leniently with empty fields");

        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.product_name, "Vinho Teste");
        assert_eq!(row.price_cents, 9900);
        assert_eq!(row.compare_at_price_cents, None);
        assert_eq!(row.stock_quantity, None);
        assert_eq!(row.weight_g, None);
        assert_eq!(row.description, None);
        assert_eq!(row.cest, None);
        assert!(row.active);
        assert_eq!(row.country.as_deref(), Some("Brasil"));
        assert_eq!(row.color.as_deref(), Some("Tinto"));
    }

    #[test]
    fn test_lenient_boolean_and_numeric_parsing() {
        let csv_data = "product_name,slug,code,price_cents,active,origem_mercadoria\n\
        W1,w-1,C-1,1000,1,0\n\
        W2,w-2,C-2,2000,0,1\n\
        W3,w-3,C-3,3000,true,0\n\
        W4,w-4,C-4,4000,false,1\n";

        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let rows: Vec<CatalogRow> = reader
            .deserialize()
            .collect::<Result<Vec<CatalogRow>, _>>()
            .expect("Should deserialize lenient booleans");

        assert_eq!(rows.len(), 4);
        assert!(rows[0].active);
        assert!(!rows[1].active);
        assert!(rows[2].active);
        assert!(!rows[3].active);
    }
