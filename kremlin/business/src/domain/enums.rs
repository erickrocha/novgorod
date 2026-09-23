use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize, ToSchema)]
pub enum Role {
    SysAdmin,
    TenantOwner,
    #[default]
    TenantUser,
    Customer,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::SysAdmin => write!(f, "SysAdmin"),
            Role::TenantOwner => write!(f, "TenantOwner"),
            Role::TenantUser => write!(f, "TenantUser"),
            Role::Customer => write!(f, "Customer"),
        }
    }
}

impl FromStr for Role {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "SysAdmin" => Ok(Self::SysAdmin),
            "TenantOwner" => Ok(Self::TenantOwner),
            "TenantUser" => Ok(Self::TenantUser),
            "Customer" => Ok(Self::Customer),
            _ => Err(format!("Invalid recommended period: {}", value))?,
        }
    }
}
