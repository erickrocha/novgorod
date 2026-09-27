use super::marketplace::PurchaseError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShippingMode {
    #[default]
    Fixed,
    Correios,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShippingService {
    pub code: String,
    pub name: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Parcel {
    pub weight_g: i64,
    pub length_mm: i64,
    pub width_mm: i64,
    pub height_mm: i64,
}
impl Parcel {
    pub fn add_item(
        &mut self,
        weight: Option<i32>,
        length: Option<i32>,
        width: Option<i32>,
        height: Option<i32>,
        quantity: i32,
    ) -> Result<(), PurchaseError> {
        let values = [weight, length, width, height];
        if quantity <= 0 || values.iter().any(|v| v.is_none_or(|v| v <= 0)) {
            return Err(PurchaseError::Validation(
                "positive SKU weight and dimensions required for Correios",
            ));
        }
        let [w, l, b, h] = values.map(|v| i64::from(v.unwrap_or_default()));
        self.weight_g = add(
            self.weight_g,
            w.checked_mul(quantity.into())
                .ok_or(PurchaseError::Validation("parcel overflow"))?,
        )?;
        self.height_mm = add(
            self.height_mm,
            h.checked_mul(quantity.into())
                .ok_or(PurchaseError::Validation("parcel overflow"))?,
        )?;
        self.length_mm = self.length_mm.max(l);
        self.width_mm = self.width_mm.max(b);
        Ok(())
    }
    pub fn packaged(mut self, allowance: &Self) -> Result<Self, PurchaseError> {
        self.weight_g = add(self.weight_g, allowance.weight_g)?;
        self.length_mm = add(self.length_mm, allowance.length_mm)?;
        self.width_mm = add(self.width_mm, allowance.width_mm)?;
        self.height_mm = add(self.height_mm, allowance.height_mm)?;
        Ok(self)
    }
    pub fn centimeters(mm: i64) -> Result<i64, PurchaseError> {
        if mm <= 0 {
            return Err(PurchaseError::Validation("positive dimensions required"));
        }
        Ok(mm / 10 + i64::from(mm % 10 != 0))
    }
}
fn add(a: i64, b: i64) -> Result<i64, PurchaseError> {
    if b < 0 {
        return Err(PurchaseError::Validation("negative packaging allowance"));
    }
    a.checked_add(b)
        .ok_or(PurchaseError::Validation("parcel overflow"))
}
pub fn normalize_cep(value: &str) -> Result<String, PurchaseError> {
    let value = value.trim();
    if !value
        .bytes()
        .all(|b| b.is_ascii_digit() || b == b'-' || b == b' ')
    {
        return Err(PurchaseError::Validation("invalid CEP"));
    }
    let digits: String = value.chars().filter(char::is_ascii_digit).collect();
    if digits.len() != 8 {
        return Err(PurchaseError::Validation("CEP must contain eight digits"));
    }
    Ok(digits)
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShippingConfig {
    pub mode: ShippingMode,
    pub origin_cep: Option<String>,
    #[serde(default)]
    pub services: Vec<ShippingService>,
    #[serde(default)]
    pub packaging: Parcel,
}
impl ShippingConfig {
    pub fn validate(&mut self) -> Result<(), PurchaseError> {
        if let Some(cep) = &self.origin_cep {
            self.origin_cep = Some(normalize_cep(cep)?);
        }
        Parcel::default().packaged(&self.packaging)?;
        let mut codes = std::collections::BTreeSet::new();
        if self.services.len() > 20
            || self.services.iter().any(|s| {
                s.code.len() != 5
                    || !s.code.bytes().all(|b| b.is_ascii_digit())
                    || s.name.trim().is_empty()
                    || s.name.len() > 100
                    || !codes.insert(&s.code)
            })
        {
            return Err(PurchaseError::Validation("invalid shipping services"));
        }
        if self.mode == ShippingMode::Correios
            && (self.origin_cep.is_none() || self.services.is_empty())
        {
            return Err(PurchaseError::Validation(
                "Correios origin and services required",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShippingOption {
    pub id: String,
    pub provider: String,
    pub service_code: String,
    pub service_name: String,
    pub price_cents: i64,
    /// Carrier transit time from posting; excludes seller preparation time.
    pub transit_days: Option<u32>,
}
pub fn sort_options(options: &mut [ShippingOption]) {
    options.sort_by(|a, b| {
        (
            a.price_cents,
            a.transit_days.unwrap_or(u32::MAX),
            &a.service_code,
        )
            .cmp(&(
                b.price_cents,
                b.transit_days.unwrap_or(u32::MAX),
                &b.service_code,
            ))
    });
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShippingSnapshot {
    pub configuration_version: i64,
    pub mode: ShippingMode,
    pub origin_cep: Option<String>,
    pub destination_cep: String,
    pub parcel: Option<Parcel>,
    /// Includes each SKU's measurements, even if changes leave the aggregate unchanged.
    pub inputs_hash: String,
    pub options: Vec<ShippingOption>,
    pub selected_option: ShippingOption,
}

#[cfg(test)]
mod tests {
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
}
