#![allow(dead_code)]

use std::str::FromStr;

use crate::endpoints::json::access_token_json::AccessTokenJson;
use crate::endpoints::json::campaign_json::{CampaignJson, CampaignTargetJson};
use crate::endpoints::json::cart_json::{CartItemJson, CartJson};
use crate::endpoints::json::catalog_json::{
    CatalogAttributeJson, CatalogAttributeValueJson, CategoryJson, ProductAttributeJson,
    ProductJson, SkuJson,
};
use crate::endpoints::json::city_json::CityJson;
use crate::endpoints::json::coupon_json::{CouponJson, CouponRedemptionJson};
use crate::endpoints::json::customer_json::{CustomerAddressJson, CustomerJson};
use crate::endpoints::json::orders_json::{OrderItemJson, OrderStatusHistoryJson, OrdersJson};
use crate::endpoints::json::product_category_json::ProductCategoryJson;
use crate::endpoints::json::product_image_json::ProductImageJson;
use crate::endpoints::json::province_json::ProvinceJson;
use crate::endpoints::json::shipping_rate_json::ShippingRateJson;
use crate::endpoints::json::sku_attribute_json::SkuAttributeValueJson;
use crate::endpoints::json::sku_stock_json::SkuStockJson;
use crate::endpoints::json::tax_rule_json::TaxRuleJson;
use crate::endpoints::json::tenant_json::TenantJson;
use crate::endpoints::json::user_json::UserJson;
use business::domain::access_token::AccessToken;
use business::domain::campaign::Campaign;
use business::domain::campaign_target::CampaignTarget;
use business::domain::cart::Cart;
use business::domain::cart_item::CartItem;
use business::domain::catalog_attribute::CatalogAttribute;
use business::domain::catalog_attribute_value::CatalogAttributeValue;
use business::domain::category::Category;
use business::domain::city::City;
use business::domain::coupon::Coupon;
use business::domain::coupon_redemption::CouponRedemption;
use business::domain::customer::Customer;
use business::domain::customer_address::CustomerAddress;
use business::domain::enums::Role;
use business::domain::order_item::OrderItem;
use business::domain::order_status_history::OrderStatusHistory;
use business::domain::orders::Orders;
use business::domain::product::Product;
use business::domain::product_attribute::ProductAttribute;
use business::domain::product_category::ProductCategory;
use business::domain::product_image::ProductImage;
use business::domain::province::Province;
use business::domain::shipping_rate::ShippingRate;
use business::domain::sku::Sku;
use business::domain::sku_attribute_value::SkuAttributeValue;
use business::domain::sku_stock::SkuStock;
use business::domain::tax_rule::TaxRule;
use business::domain::tenant::Tenant;
use business::domain::user::User;

pub trait Mapper<T, U> {
    fn json(t: T) -> U;

    fn domain(u: U) -> T;

    fn json_vec(u: Vec<T>) -> Vec<U> {
        u.into_iter().map(Self::json).collect()
    }
}

pub struct AccessTokenMapper {}

impl Mapper<AccessToken, AccessTokenJson> for AccessTokenMapper {
    fn json(access_token: AccessToken) -> AccessTokenJson {
        AccessTokenJson {
            access_token: access_token.access_token,
            token_type: access_token.token_type,
            expire_in: access_token.expire_in,
            refresh_token: access_token.refresh_token,
            email: access_token.email,
            uuid: access_token.uuid,
            name: access_token.name,
            user_id: access_token.user_id,
            role: access_token.role,
            tenant_id: access_token.tenant_id,
            first_login: access_token.first_login,
        }
    }

    fn domain(u: AccessTokenJson) -> AccessToken {
        AccessToken {
            access_token: u.access_token,
            token_type: u.token_type,
            expire_in: u.expire_in,
            refresh_token: u.refresh_token,
            email: u.email,
            uuid: u.uuid,
            name: u.name,
            user_id: u.user_id,
            role: u.role,
            tenant_id: u.tenant_id,
            first_login: u.first_login,
        }
    }
}

pub struct UserMapper {}

impl Mapper<User, UserJson> for UserMapper {
    fn json(user: User) -> UserJson {
        UserJson {
            id: user.id,
            uuid: user.uuid,
            name: user.name,
            email: user.email,
            password: None,
            enabled: user.enabled,
            first_login: user.first_login,
            role: user.role.to_string(),
            tenant_id: user.tenant_id,
            created_at: user.created_at,
            created_by: user.created_by,
            updated_at: user.updated_at,
            updated_by: user.updated_by,
        }
    }

    fn domain(u: UserJson) -> User {
        User {
            id: u.id,
            uuid: u.uuid,
            email: u.email,
            name: u.name,
            password: u.password.unwrap_or_default(),
            enabled: u.enabled,
            first_login: u.first_login,
            role: Role::from_str(&u.role).unwrap(),
            tenant_id: u.tenant_id,
            created_at: u.created_at,
            created_by: u.created_by,
            updated_at: u.updated_at,
            updated_by: u.updated_by,
        }
    }
}

pub struct TenantMapper {}

fn optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn canonical_address(canonical: Option<String>, alias: Option<String>) -> Option<String> {
    optional_text(canonical).or_else(|| optional_text(alias))
}

fn country_code(value: Option<String>) -> Option<String> {
    optional_text(value).map(|value| value.to_uppercase())
}

impl Mapper<Tenant, TenantJson> for TenantMapper {
    fn json(t: Tenant) -> TenantJson {
        let province = t.administrative_area.clone();
        let city = t.locality.clone();
        let zipcode = t.postal_code.clone();
        TenantJson {
            id: t.id,
            uuid: t.uuid,
            business_name: Some(t.business_name),
            company_name: t.company_name,
            tax_id: Some(t.tax_id),
            email: t.email,
            phone: t.phone,
            web_site: t.web_site,
            address_line1: t.address_line1,
            address_line2: t.address_line2,
            locality: t.locality,
            administrative_area: t.administrative_area,
            postal_code: t.postal_code,
            country_code: t.country_code,
            province,
            city,
            zipcode,
            created_at: t.created_at,
            created_by: t.created_by,
            updated_at: t.updated_at,
            updated_by: t.updated_by,
        }
    }

    fn domain(u: TenantJson) -> Tenant {
        let locality = canonical_address(u.locality, u.city);
        let administrative_area = canonical_address(u.administrative_area, u.province);
        let postal_code = canonical_address(u.postal_code, u.zipcode);
        Tenant {
            id: u.id,
            uuid: u.uuid,
            business_name: u.business_name.unwrap(),
            company_name: u.company_name,
            tax_id: u.tax_id.unwrap(),
            email: optional_text(u.email),
            phone: optional_text(u.phone),
            web_site: u.web_site,
            address_line1: optional_text(u.address_line1),
            address_line2: optional_text(u.address_line2),
            locality,
            administrative_area,
            postal_code,
            country_code: country_code(u.country_code),
            created_at: u.created_at,
            created_by: u.created_by,
            updated_at: u.updated_at,
            updated_by: u.updated_by,
        }
    }
}

pub struct ProvinceMapper;
impl Mapper<Province, ProvinceJson> for ProvinceMapper {
    fn json(t: Province) -> ProvinceJson {
        ProvinceJson {
            id: t.id,
            uuid: t.uuid,
            acronym: t.acronym,
            name: t.name,
            ibge_code: t.ibge_code,
            country_code: t.country_code,
        }
    }

    fn domain(u: ProvinceJson) -> Province {
        Province {
            id: u.id,
            uuid: u.uuid,
            acronym: u.acronym,
            name: u.name,
            country_code: u.country_code,
            ibge_code: u.ibge_code,
        }
    }
}

pub struct CityMapper;
impl Mapper<City, CityJson> for CityMapper {
    fn json(t: City) -> CityJson {
        CityJson {
            id: t.id,
            uuid: t.uuid,
            province_id: t.province_id,
            name: t.name,
            ibge_code: t.ibge_code,
        }
    }

    fn domain(u: CityJson) -> City {
        City {
            id: u.id,
            uuid: u.uuid,
            province_id: u.province_id,
            name: u.name,
            ibge_code: u.ibge_code,
        }
    }
}

pub struct ProductImageMapper;

impl Mapper<ProductImage, ProductImageJson> for ProductImageMapper {
    fn json(t: ProductImage) -> ProductImageJson {
        ProductImageJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            product_id: t.product_id,
            sku_id: t.sku_id,
            alt_text: t.alt_text,
            sort_order: t.sort_order,
            is_primary: t.is_primary,
            storage_provider: t.storage_provider,
            bucket: t.bucket,
            object_key: t.object_key,
            original_filename: t.original_filename,
            mime_type: t.mime_type,
            size_bytes: t.size_bytes,
            storage_status: t.storage_status,
            cdn_url: t.cdn_url,
        }
    }

    fn domain(u: ProductImageJson) -> ProductImage {
        ProductImage {
            id: Some(u.id),
            uuid: Some(u.uuid),
            tenant_id: u.tenant_id,
            product_id: u.product_id,
            sku_id: u.sku_id,
            alt_text: u.alt_text,
            sort_order: u.sort_order,
            is_primary: u.is_primary,
            storage_provider: u.storage_provider,
            bucket: u.bucket,
            object_key: u.object_key,
            storage_identity_hash: None,
            object_version: None,
            etag: None,
            checksum_sha256: None,
            original_filename: u.original_filename,
            mime_type: u.mime_type,
            size_bytes: u.size_bytes,
            width_px: None,
            height_px: None,
            storage_status: u.storage_status,
            cdn_url: u.cdn_url,
        }
    }
}

pub struct CategoryMapper;
impl Mapper<Category, CategoryJson> for CategoryMapper {
    fn json(t: Category) -> CategoryJson {
        CategoryJson {
            id: t.id,
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            name: t.name,
            slug: t.slug,
            parent_id: t.parent_id,
            active: t.active,
        }
    }

    fn domain(u: CategoryJson) -> Category {
        Category {
            id: u.id,
            uuid: Some(u.uuid),
            tenant_id: u.tenant_id,
            name: u.name,
            slug: u.slug,
            parent_id: u.parent_id,
            active: u.active,
        }
    }
}

pub struct ProductMapper;
impl Mapper<Product, ProductJson> for ProductMapper {
    fn json(t: Product) -> ProductJson {
        ProductJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            name: t.name,
            slug: t.slug,
            description: t.description,
            brand: t.brand,
            active: t.active,
            ncm: t.ncm,
            cest: t.cest,
            origem_mercadoria: t.origem_mercadoria,
        }
    }

    fn domain(u: ProductJson) -> Product {
        Product {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            name: u.name,
            slug: u.slug,
            description: u.description,
            brand: u.brand,
            active: u.active,
            ncm: u.ncm,
            cest: u.cest,
            origem_mercadoria: u.origem_mercadoria,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }
}

pub struct SkuMapper;
impl Mapper<Sku, SkuJson> for SkuMapper {
    fn json(t: Sku) -> SkuJson {
        SkuJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            product_id: t.product_id,
            code: t.code,
            variant_key: t.variant_key,
            price_cents: t.price_cents,
            compare_at_price_cents: t.compare_at_price_cents,
            weight_g: t.weight_g,
            width_mm: t.width_mm,
            height_mm: t.height_mm,
            length_mm: t.length_mm,
            active: t.active,
        }
    }

    fn domain(u: SkuJson) -> Sku {
        Sku {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            product_id: u.product_id,
            code: u.code,
            variant_key: u.variant_key,
            price_cents: u.price_cents,
            compare_at_price_cents: u.compare_at_price_cents,
            weight_g: u.weight_g,
            width_mm: u.width_mm,
            height_mm: u.height_mm,
            length_mm: u.length_mm,
            active: u.active,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }
}

pub struct CatalogAttributeMapper;
impl Mapper<CatalogAttribute, CatalogAttributeJson> for CatalogAttributeMapper {
    fn json(t: CatalogAttribute) -> CatalogAttributeJson {
        CatalogAttributeJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            name: t.name,
            display_type: t.display_type,
        }
    }

    fn domain(u: CatalogAttributeJson) -> CatalogAttribute {
        CatalogAttribute {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            name: u.name,
            display_type: u.display_type,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }
}

pub struct CatalogAttributeValueMapper;
impl Mapper<CatalogAttributeValue, CatalogAttributeValueJson> for CatalogAttributeValueMapper {
    fn json(t: CatalogAttributeValue) -> CatalogAttributeValueJson {
        CatalogAttributeValueJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            attribute_id: t.attribute_id,
            value: t.value,
        }
    }

    fn domain(u: CatalogAttributeValueJson) -> CatalogAttributeValue {
        CatalogAttributeValue {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            attribute_id: u.attribute_id,
            value: u.value,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }
}

pub struct ProductAttributeMapper;
impl Mapper<ProductAttribute, ProductAttributeJson> for ProductAttributeMapper {
    fn json(t: ProductAttribute) -> ProductAttributeJson {
        ProductAttributeJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            product_id: t.product_id,
            attribute_id: t.attribute_id,
            required: t.required,
            sort_order: t.sort_order,
        }
    }

    fn domain(u: ProductAttributeJson) -> ProductAttribute {
        ProductAttribute {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            product_id: u.product_id,
            attribute_id: u.attribute_id,
            required: u.required,
            sort_order: u.sort_order,
            created_at: None,
            created_by: None,
        }
    }
}

pub struct ShippingRateMapper;
impl Mapper<ShippingRate, ShippingRateJson> for ShippingRateMapper {
    fn json(t: ShippingRate) -> ShippingRateJson {
        ShippingRateJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            uf: t.uf,
            price_cents: t.price_cents,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: ShippingRateJson) -> ShippingRate {
        ShippingRate {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            uf: u.uf,
            price_cents: u.price_cents,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct CustomerMapper;
impl Mapper<Customer, CustomerJson> for CustomerMapper {
    fn json(t: Customer) -> CustomerJson {
        CustomerJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            name: t.name,
            email: t.email,
            cpf: t.cpf,
            phone: t.phone,
            marketing_consent: t.marketing_consent,
            active: t.active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: CustomerJson) -> Customer {
        Customer {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            name: u.name,
            email: u.email,
            password_hash: String::new(),
            cpf: u.cpf,
            phone: u.phone,
            marketing_consent: u.marketing_consent,
            consent_at: None,
            active: u.active,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct CustomerAddressMapper;
impl Mapper<CustomerAddress, CustomerAddressJson> for CustomerAddressMapper {
    fn json(t: CustomerAddress) -> CustomerAddressJson {
        CustomerAddressJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            customer_id: t.customer_id,
            label: t.label,
            recipient: t.recipient,
            cep: t.cep,
            logradouro: t.logradouro,
            numero: t.numero,
            complemento: t.complemento,
            bairro: t.bairro,
            cidade: t.cidade,
            uf: t.uf,
            is_default: t.is_default,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: CustomerAddressJson) -> CustomerAddress {
        CustomerAddress {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            customer_id: u.customer_id,
            label: u.label,
            recipient: u.recipient,
            cep: u.cep,
            logradouro: u.logradouro,
            numero: u.numero,
            complemento: u.complemento,
            bairro: u.bairro,
            cidade: u.cidade,
            uf: u.uf,
            is_default: u.is_default,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct TaxRuleMapper;
impl Mapper<TaxRule, TaxRuleJson> for TaxRuleMapper {
    fn json(t: TaxRule) -> TaxRuleJson {
        TaxRuleJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            uf_origem: t.uf_origem,
            uf_destino: t.uf_destino,
            ncm_prefix: t.ncm_prefix,
            regime: t.regime,
            csosn: t.csosn,
            cfop: t.cfop,
            icms_rate_bp: t.icms_rate_bp,
            active: t.active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: TaxRuleJson) -> TaxRule {
        TaxRule {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            uf_origem: u.uf_origem,
            uf_destino: u.uf_destino,
            ncm_prefix: u.ncm_prefix,
            regime: u.regime,
            csosn: u.csosn,
            cfop: u.cfop,
            icms_rate_bp: u.icms_rate_bp,
            active: u.active,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct CampaignMapper;
impl Mapper<Campaign, CampaignJson> for CampaignMapper {
    fn json(t: Campaign) -> CampaignJson {
        CampaignJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            name: t.name,
            campaign_type: t.campaign_type,
            value: t.value,
            scope: t.scope,
            starts_at: t.starts_at,
            ends_at: t.ends_at,
            active: t.active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: CampaignJson) -> Campaign {
        Campaign {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            name: u.name,
            campaign_type: u.campaign_type,
            value: u.value,
            scope: u.scope,
            starts_at: u.starts_at,
            ends_at: u.ends_at,
            active: u.active,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct CampaignTargetMapper;
impl Mapper<CampaignTarget, CampaignTargetJson> for CampaignTargetMapper {
    fn json(t: CampaignTarget) -> CampaignTargetJson {
        CampaignTargetJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            campaign_id: t.campaign_id,
            target_type: t.target_type,
            target_id: t.target_id,
            created_at: t.created_at,
        }
    }

    fn domain(u: CampaignTargetJson) -> CampaignTarget {
        CampaignTarget {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            campaign_id: u.campaign_id,
            target_type: u.target_type,
            target_id: u.target_id,
            created_at: u.created_at,
            created_by: None,
        }
    }
}

pub struct CouponMapper;
impl Mapper<Coupon, CouponJson> for CouponMapper {
    fn json(t: Coupon) -> CouponJson {
        CouponJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            code: t.code,
            campaign_id: t.campaign_id,
            coupon_type: t.coupon_type,
            value: t.value,
            min_order_cents: t.min_order_cents,
            max_uses: t.max_uses,
            max_uses_per_customer: t.max_uses_per_customer,
            starts_at: t.starts_at,
            expires_at: t.expires_at,
            active: t.active,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: CouponJson) -> Coupon {
        Coupon {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            code: u.code,
            campaign_id: u.campaign_id,
            coupon_type: u.coupon_type,
            value: u.value,
            min_order_cents: u.min_order_cents,
            max_uses: u.max_uses,
            max_uses_per_customer: u.max_uses_per_customer,
            starts_at: u.starts_at,
            expires_at: u.expires_at,
            active: u.active,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct CouponRedemptionMapper;
impl Mapper<CouponRedemption, CouponRedemptionJson> for CouponRedemptionMapper {
    fn json(t: CouponRedemption) -> CouponRedemptionJson {
        CouponRedemptionJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            coupon_id: t.coupon_id,
            order_id: t.order_id,
            customer_id: t.customer_id,
            created_at: t.created_at,
        }
    }

    fn domain(u: CouponRedemptionJson) -> CouponRedemption {
        CouponRedemption {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            coupon_id: u.coupon_id,
            order_id: u.order_id,
            customer_id: u.customer_id,
            created_at: u.created_at,
            created_by: None,
        }
    }
}

pub struct CartMapper;
impl Mapper<Cart, CartJson> for CartMapper {
    fn json(t: Cart) -> CartJson {
        CartJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            customer_id: t.customer_id,
            status: t.status,
            expires_at: t.expires_at,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: CartJson) -> Cart {
        Cart {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            customer_id: u.customer_id,
            status: u.status,
            expires_at: u.expires_at,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct CartItemMapper;
impl Mapper<CartItem, CartItemJson> for CartItemMapper {
    fn json(t: CartItem) -> CartItemJson {
        CartItemJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            cart_id: t.cart_id,
            sku_id: t.sku_id,
            quantity: t.quantity,
            unit_price_cents: t.unit_price_cents,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: CartItemJson) -> CartItem {
        CartItem {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            cart_id: u.cart_id,
            sku_id: u.sku_id,
            quantity: u.quantity,
            unit_price_cents: u.unit_price_cents,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct OrdersMapper;
impl Mapper<Orders, OrdersJson> for OrdersMapper {
    fn json(t: Orders) -> OrdersJson {
        OrdersJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            number: t.number,
            customer_id: t.customer_id,
            status: t.status,
            payment_status: t.payment_status,
            subtotal_cents: t.subtotal_cents,
            discount_cents: t.discount_cents,
            shipping_cents: t.shipping_cents,
            tax_total_cents: t.tax_total_cents,
            total_cents: t.total_cents,
            coupon_id: t.coupon_id,
            coupon_code: t.coupon_code,
            ship_recipient: t.ship_recipient,
            ship_cep: t.ship_cep,
            ship_logradouro: t.ship_logradouro,
            ship_numero: t.ship_numero,
            ship_complemento: t.ship_complemento,
            ship_bairro: t.ship_bairro,
            ship_cidade: t.ship_cidade,
            ship_uf: t.ship_uf,
            placed_at: Some(t.placed_at),
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: OrdersJson) -> Orders {
        let now = chrono::Utc::now().naive_utc();
        Orders {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            number: u.number,
            customer_id: u.customer_id,
            status: u.status,
            payment_status: u.payment_status,
            subtotal_cents: u.subtotal_cents,
            discount_cents: u.discount_cents,
            shipping_cents: u.shipping_cents,
            tax_total_cents: u.tax_total_cents,
            total_cents: u.total_cents,
            coupon_id: u.coupon_id,
            coupon_code: u.coupon_code,
            ship_recipient: u.ship_recipient,
            ship_cep: u.ship_cep,
            ship_logradouro: u.ship_logradouro,
            ship_numero: u.ship_numero,
            ship_complemento: u.ship_complemento,
            ship_bairro: u.ship_bairro,
            ship_cidade: u.ship_cidade,
            ship_uf: u.ship_uf,
            placed_at: u.placed_at.unwrap_or(now),
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct OrderItemMapper;
impl Mapper<OrderItem, OrderItemJson> for OrderItemMapper {
    fn json(t: OrderItem) -> OrderItemJson {
        OrderItemJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            order_id: t.order_id,
            sku_id: t.sku_id,
            sku_code: t.sku_code,
            product_name: t.product_name,
            attributes_desc: t.attributes_desc,
            quantity: t.quantity,
            unit_price_cents: t.unit_price_cents,
            discount_cents: t.discount_cents,
            ncm: t.ncm,
            cfop: t.cfop,
            csosn: t.csosn,
            icms_rate_bp: t.icms_rate_bp,
            tax_cents: t.tax_cents,
            total_cents: t.total_cents,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn domain(u: OrderItemJson) -> OrderItem {
        OrderItem {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            order_id: u.order_id,
            sku_id: u.sku_id,
            sku_code: u.sku_code,
            product_name: u.product_name,
            attributes_desc: u.attributes_desc,
            quantity: u.quantity,
            unit_price_cents: u.unit_price_cents,
            discount_cents: u.discount_cents,
            ncm: u.ncm,
            cfop: u.cfop,
            csosn: u.csosn,
            icms_rate_bp: u.icms_rate_bp,
            tax_cents: u.tax_cents,
            total_cents: u.total_cents,
            created_at: u.created_at,
            created_by: None,
            updated_at: u.updated_at,
            updated_by: None,
        }
    }
}

pub struct OrderStatusHistoryMapper;
impl Mapper<OrderStatusHistory, OrderStatusHistoryJson> for OrderStatusHistoryMapper {
    fn json(t: OrderStatusHistory) -> OrderStatusHistoryJson {
        OrderStatusHistoryJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            order_id: t.order_id,
            from_status: t.from_status,
            to_status: t.to_status,
            actor_type: t.actor_type,
            actor_id: t.actor_id,
            note: t.note,
            created_at: t.created_at,
        }
    }

    fn domain(u: OrderStatusHistoryJson) -> OrderStatusHistory {
        OrderStatusHistory {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            order_id: u.order_id,
            from_status: u.from_status,
            to_status: u.to_status,
            actor_type: u.actor_type,
            actor_id: u.actor_id,
            note: u.note,
            created_at: u.created_at,
            created_by: None,
        }
    }
}

pub struct ProductCategoryMapper;
impl Mapper<ProductCategory, ProductCategoryJson> for ProductCategoryMapper {
    fn json(t: ProductCategory) -> ProductCategoryJson {
        ProductCategoryJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            product_id: t.product_id,
            category_id: t.category_id,
            is_primary: t.is_primary,
            created_at: t.created_at,
            created_by: t.created_by,
        }
    }

    fn domain(u: ProductCategoryJson) -> ProductCategory {
        ProductCategory {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            product_id: u.product_id,
            category_id: u.category_id,
            is_primary: u.is_primary,
            created_at: u.created_at,
            created_by: u.created_by,
        }
    }
}

pub struct SkuAttributeValueMapper;
pub type SkuAttributeMapper = SkuAttributeValueMapper;

impl Mapper<SkuAttributeValue, SkuAttributeValueJson> for SkuAttributeValueMapper {
    fn json(t: SkuAttributeValue) -> SkuAttributeValueJson {
        SkuAttributeValueJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            product_id: t.product_id,
            sku_id: t.sku_id,
            product_attribute_id: t.product_attribute_id,
            attribute_id: t.attribute_id,
            attribute_value_id: t.attribute_value_id,
            created_at: t.created_at,
            created_by: t.created_by,
            updated_at: t.updated_at,
            updated_by: t.updated_by,
        }
    }

    fn domain(u: SkuAttributeValueJson) -> SkuAttributeValue {
        SkuAttributeValue {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            product_id: u.product_id,
            sku_id: u.sku_id,
            product_attribute_id: u.product_attribute_id,
            attribute_id: u.attribute_id,
            attribute_value_id: u.attribute_value_id,
            created_at: u.created_at,
            created_by: u.created_by,
            updated_at: u.updated_at,
            updated_by: u.updated_by,
        }
    }
}

pub struct SkuStockMapper;
impl Mapper<SkuStock, SkuStockJson> for SkuStockMapper {
    fn json(t: SkuStock) -> SkuStockJson {
        SkuStockJson {
            id: t.id.unwrap_or_default(),
            uuid: t.uuid.unwrap_or_default(),
            tenant_id: t.tenant_id,
            sku_id: t.sku_id,
            quantity: t.quantity,
            reserved: t.reserved,
            created_at: t.created_at,
            created_by: t.created_by,
            updated_at: t.updated_at,
            updated_by: t.updated_by,
        }
    }

    fn domain(u: SkuStockJson) -> SkuStock {
        SkuStock {
            id: if u.id > 0 { Some(u.id) } else { None },
            uuid: if u.uuid.is_empty() { None } else { Some(u.uuid) },
            tenant_id: u.tenant_id,
            sku_id: u.sku_id,
            quantity: u.quantity,
            reserved: u.reserved,
            created_at: u.created_at,
            created_by: u.created_by,
            updated_at: u.updated_at,
            updated_by: u.updated_by,
        }
    }
}

#[cfg(test)]
mod address_mapping_tests {
    use super::{canonical_address, country_code, optional_text};

    #[test]
    fn canonical_address_wins_and_legacy_alias_fills_missing_value() {
        assert_eq!(
            canonical_address(Some("  Campinas ".into()), Some("São Paulo".into())),
            Some("Campinas".into())
        );
        assert_eq!(
            canonical_address(Some("  ".into()), Some(" São Paulo ".into())),
            Some("São Paulo".into())
        );
    }

    #[test]
    fn optional_address_values_are_cleaned_and_country_is_uppercase() {
        assert_eq!(optional_text(Some("  ".into())), None);
        assert_eq!(country_code(Some(" br ".into())), Some("BR".into()));
    }
}

#[cfg(test)]
mod new_domain_mapper_tests {
    use super::*;

    #[test]
    fn test_shipping_rate_mapper() {
        let now = chrono::Utc::now().naive_utc();
        let domain = business::domain::shipping_rate::ShippingRate {
            id: Some(1),
            uuid: Some("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(10),
            uf: "SP".to_string(),
            price_cents: 2500,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: None,
            updated_by: None,
        };

        let json = ShippingRateMapper::json(domain.clone());
        assert_eq!(json.uf, "SP");
        assert_eq!(json.price_cents, 2500);
        assert_eq!(json.tenant_id, Some(10));

        let back_to_domain = ShippingRateMapper::domain(json);
        assert_eq!(back_to_domain.uf, domain.uf);
        assert_eq!(back_to_domain.price_cents, domain.price_cents);
    }

    #[test]
    fn test_customer_mapper() {
        let now = chrono::Utc::now().naive_utc();
        let domain = business::domain::customer::Customer {
            id: Some(42),
            uuid: Some("b1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(5),
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: "secret_hash".to_string(),
            cpf: Some("12345678901".to_string()),
            phone: Some("+5511999999999".to_string()),
            marketing_consent: true,
            consent_at: None,
            active: true,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: None,
            updated_by: None,
        };

        let json = CustomerMapper::json(domain.clone());
        assert_eq!(json.name, "Alice Smith");
        assert_eq!(json.email, "alice@example.com");
        assert_eq!(json.cpf, Some("12345678901".to_string()));

        let back = CustomerMapper::domain(json);
        assert_eq!(back.name, domain.name);
        assert_eq!(back.email, domain.email);
        assert_eq!(back.cpf, domain.cpf);
    }

    #[test]
    fn test_coupon_mapper() {
        let now = chrono::Utc::now().naive_utc();
        let domain = business::domain::coupon::Coupon {
            id: Some(7),
            uuid: Some("c1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            campaign_id: Some(2),
            code: "SUMMER10".to_string(),
            coupon_type: "percentage".to_string(),
            value: 10,
            min_order_cents: None,
            max_uses: Some(100),
            max_uses_per_customer: Some(1),
            starts_at: None,
            expires_at: None,
            active: true,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: None,
            updated_by: None,
        };

        let json = CouponMapper::json(domain.clone());
        assert_eq!(json.code, "SUMMER10");
        assert_eq!(json.coupon_type, "percentage");
        assert_eq!(json.value, 10);

        let back = CouponMapper::domain(json);
        assert_eq!(back.code, domain.code);
        assert_eq!(back.coupon_type, domain.coupon_type);
    }

    #[test]
    fn test_product_category_mapper() {
        let domain = ProductCategory {
            id: Some(15),
            uuid: Some("d1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            product_id: 100,
            category_id: 200,
            is_primary: true,
            created_at: None,
            created_by: None,
        };
        let json = ProductCategoryMapper::json(domain.clone());
        assert_eq!(json.id, 15);
        assert_eq!(json.product_id, 100);
        assert_eq!(json.category_id, 200);
        assert!(json.is_primary);

        let back = ProductCategoryMapper::domain(json);
        assert_eq!(back.id, domain.id);
        assert_eq!(back.product_id, domain.product_id);
    }

    #[test]
    fn test_sku_attribute_value_mapper() {
        let domain = SkuAttributeValue {
            id: Some(25),
            uuid: Some("e1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            product_id: 10,
            sku_id: 20,
            product_attribute_id: 30,
            attribute_id: 40,
            attribute_value_id: 50,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let json = SkuAttributeValueMapper::json(domain.clone());
        assert_eq!(json.sku_id, 20);
        assert_eq!(json.attribute_value_id, 50);

        let back = SkuAttributeValueMapper::domain(json);
        assert_eq!(back.sku_id, domain.sku_id);
        assert_eq!(back.attribute_value_id, domain.attribute_value_id);
    }

    #[test]
    fn test_sku_stock_mapper() {
        let domain = SkuStock {
            id: Some(35),
            uuid: Some("f1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8".to_string()),
            tenant_id: Some(1),
            sku_id: 20,
            quantity: 150,
            reserved: 10,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        let json = SkuStockMapper::json(domain.clone());
        assert_eq!(json.quantity, 150);
        assert_eq!(json.reserved, 10);

        let back = SkuStockMapper::domain(json);
        assert_eq!(back.quantity, domain.quantity);
        assert_eq!(back.reserved, domain.reserved);
    }
}


