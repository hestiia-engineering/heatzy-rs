use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::HeatzyError;

/// Login credentials
#[derive(Debug, Serialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub uid: String,
    pub expire_at: i64,
}

/// Device information
#[derive(Debug, Clone, Deserialize)]
pub struct Device {
    pub did: String,
    pub dev_alias: Option<String>,
    pub product_name: String,
    pub mac: String,
    pub is_online: bool,
}

/// Internal structure for parsing device list response
#[derive(Debug, Deserialize)]
pub(crate) struct DevicesResponse {
    pub devices: Vec<Device>,
}

/// Internal structure for parsing device data
#[derive(Debug, Deserialize)]
pub(crate) struct DeviceDataResponse {
    pub attr: DeviceAttributes,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceAttributes {
    pub mode: serde_json::Value, // Can be string or number
}

/// Device heating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMode {
    Comfort,         // 0, "cft"
    Eco,             // 1, "eco"
    FrostProtection, // 2, "fro"
    Stop,            // 3, "stop"
    ComfortMinus1,   // 4, "cft1"
    ComfortMinus2,   // 5, "cft2"
}

impl DeviceMode {
    /// Convert from API integer value
    pub fn from_int(value: i32) -> Result<Self, HeatzyError> {
        match value {
            0 => Ok(DeviceMode::Comfort),
            1 => Ok(DeviceMode::Eco),
            2 => Ok(DeviceMode::FrostProtection),
            3 => Ok(DeviceMode::Stop),
            4 => Ok(DeviceMode::ComfortMinus1),
            5 => Ok(DeviceMode::ComfortMinus2),
            _ => Err(HeatzyError::InvalidMode(format!("Invalid mode number: {}", value))),
        }
    }
    
    /// Convert from API string value
    pub fn from_str_api(value: &str) -> Result<Self, HeatzyError> {
        match value {
            "cft" => Ok(DeviceMode::Comfort),
            "eco" => Ok(DeviceMode::Eco),
            "fro" => Ok(DeviceMode::FrostProtection),
            "stop" => Ok(DeviceMode::Stop),
            "cft1" => Ok(DeviceMode::ComfortMinus1),
            "cft2" => Ok(DeviceMode::ComfortMinus2),
            _ => Err(HeatzyError::InvalidMode(format!("Invalid mode string: {}", value))),
        }
    }
    
    /// Convert from CLI string value
    pub fn from_cli_str(value: &str) -> Result<Self, HeatzyError> {
        match value.to_lowercase().as_str() {
            "comfort" => Ok(DeviceMode::Comfort),
            "eco" => Ok(DeviceMode::Eco),
            "frost-protection" | "frost" => Ok(DeviceMode::FrostProtection),
            "stop" => Ok(DeviceMode::Stop),
            "comfort-1" | "comfort-minus-1" => Ok(DeviceMode::ComfortMinus1),
            "comfort-2" | "comfort-minus-2" => Ok(DeviceMode::ComfortMinus2),
            _ => Err(HeatzyError::InvalidMode(format!("Invalid mode: {}. Valid modes are: comfort, eco, frost-protection, stop, comfort-1, comfort-2", value))),
        }
    }
    
    /// Convert to API integer value
    pub fn to_int(&self) -> i32 {
        match self {
            DeviceMode::Comfort => 0,
            DeviceMode::Eco => 1,
            DeviceMode::FrostProtection => 2,
            DeviceMode::Stop => 3,
            DeviceMode::ComfortMinus1 => 4,
            DeviceMode::ComfortMinus2 => 5,
        }
    }
    
    /// Convert to API string value
    pub fn to_str_api(&self) -> &'static str {
        match self {
            DeviceMode::Comfort => "cft",
            DeviceMode::Eco => "eco",
            DeviceMode::FrostProtection => "fro",
            DeviceMode::Stop => "stop",
            DeviceMode::ComfortMinus1 => "cft1",
            DeviceMode::ComfortMinus2 => "cft2",
        }
    }
    
    /// Get CLI-friendly string representation
    pub fn to_cli_str(&self) -> &'static str {
        match self {
            DeviceMode::Comfort => "comfort",
            DeviceMode::Eco => "eco",
            DeviceMode::FrostProtection => "frost-protection",
            DeviceMode::Stop => "stop",
            DeviceMode::ComfortMinus1 => "comfort-1",
            DeviceMode::ComfortMinus2 => "comfort-2",
        }
    }
}

impl fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_cli_str())
    }
}

/// Control attributes for setting device mode
#[derive(Debug, Serialize)]
pub(crate) struct ControlRequest {
    pub attrs: ControlAttributes,
}

#[derive(Debug, Serialize)]
pub(crate) struct ControlAttributes {
    pub mode: i32,
}