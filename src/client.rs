use crate::error::HeatzyError;
use crate::models::*;
use log::{debug, info, trace};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::time::Duration;

const BASE_URL: &str = "https://euapi.gizwits.com/app";
const APP_ID: &str = "c70a66ff039d41b4a220e198b0fcc8b3";
const APP_ID_HEADER: &str = "X-Gizwits-Application-Id";
const USER_TOKEN_HEADER: &str = "X-Gizwits-User-token";

/// Heatzy API client
pub struct Client {
    http_client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl Client {
    /// Create a new Heatzy client
    pub fn new() -> Result<Self, HeatzyError> {
        let mut headers = HeaderMap::new();
        headers.insert(APP_ID_HEADER, HeaderValue::from_static(APP_ID));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        let http_client = reqwest::Client::builder()
            .use_rustls_tls()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            http_client,
            base_url: BASE_URL.to_string(),
            token: None,
        })
    }
    
    /// Login to the API and return the authentication response
    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse, HeatzyError> {
        info!("Logging in to Heatzy API");
        
        let url = format!("{}/login", self.base_url);
        let credentials = LoginCredentials {
            username: username.to_string(),
            password: password.to_string(),
        };
        
        debug!("Sending login request");
        let response = self.http_client
            .post(&url)
            .json(&credentials)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeatzyError::Auth(format!("Login failed with status {}: {}", status, error_text)));
        }
        
        let auth_response: AuthResponse = response.json().await?;
        info!("Successfully authenticated");
        debug!("Token expires at: {}", auth_response.expire_at);
        
        Ok(auth_response)
    }
    
    /// Connect to the API with username and password (login and set token)
    pub async fn connect(&mut self, username: &str, password: &str) -> Result<(), HeatzyError> {
        let auth_response = self.login(username, password).await?;
        self.set_token(auth_response.token);
        Ok(())
    }
    
    /// Set the authentication token manually
    pub fn set_token(&mut self, token: String) {
        debug!("Setting token manually");
        self.token = Some(token);
    }
    
    /// List all devices
    pub async fn list_devices(&self) -> Result<Vec<Device>, HeatzyError> {
        self.ensure_authenticated()?;
        info!("Listing devices");
        
        let url = format!("{}/bindings?limit=100&skip=0", self.base_url);
        let response = self.authenticated_get(&url).await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeatzyError::Api(format!("Failed to list devices with status {}: {}", status, error_text)));
        }
        
        let devices_response: DevicesResponse = response.json().await?;
        info!("Found {} devices", devices_response.devices.len());
        
        for device in &devices_response.devices {
            debug!("Device: {} ({})", device.dev_alias, device.did);
        }
        
        Ok(devices_response.devices)
    }
    
    /// Get a device by name
    pub async fn get_device_by_name(&self, name: &str) -> Result<Device, HeatzyError> {
        info!("Looking for device with name: {}", name);
        let devices = self.list_devices().await?;
        
        devices
            .into_iter()
            .find(|d| d.dev_alias == name)
            .ok_or_else(|| HeatzyError::NotFound(format!("Device with name '{}' not found", name)))
    }
    
    /// Get device information by ID
    pub async fn get_device(&self, device_id: &str) -> Result<Device, HeatzyError> {
        self.ensure_authenticated()?;
        info!("Getting device info for: {}", device_id);
        
        let url = format!("{}/devices/{}", self.base_url, device_id);
        let response = self.authenticated_get(&url).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(HeatzyError::NotFound(format!("Device '{}' not found", device_id)));
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeatzyError::Api(format!("Failed to get device with status {}: {}", status, error_text)));
        }
        
        let device: Device = response.json().await?;
        Ok(device)
    }
    
    /// Get the current mode of a device
    pub async fn get_device_mode(&self, device_id: &str) -> Result<DeviceMode, HeatzyError> {
        self.ensure_authenticated()?;
        info!("Getting mode for device: {}", device_id);
        
        let url = format!("{}/devdata/{}/latest", self.base_url, device_id);
        let response = self.authenticated_get(&url).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(HeatzyError::NotFound(format!("Device '{}' not found", device_id)));
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeatzyError::Api(format!("Failed to get device data with status {}: {}", status, error_text)));
        }
        
        let device_data: DeviceDataResponse = response.json().await?;
        let mode_value = &device_data.attr.mode;
        
        trace!("Raw mode value: {:?}", mode_value);
        
        // Try to parse as number first, then as string
        let mode = if let Some(num) = mode_value.as_i64() {
            DeviceMode::from_int(num as i32)?
        } else if let Some(s) = mode_value.as_str() {
            DeviceMode::from_str_api(s)?
        } else {
            return Err(HeatzyError::Api(format!("Invalid mode value: {:?}", mode_value)));
        };
        
        info!("Device mode: {}", mode);
        Ok(mode)
    }
    
    /// Set the mode of a device
    pub async fn set_device_mode(&self, device_id: &str, mode: DeviceMode) -> Result<(), HeatzyError> {
        self.ensure_authenticated()?;
        info!("Setting mode for device {} to {}", device_id, mode);
        
        let url = format!("{}/control/{}", self.base_url, device_id);
        let control_request = ControlRequest {
            attrs: ControlAttributes {
                mode: mode.to_int(),
            },
        };
        
        debug!("Sending control request with mode: {}", mode.to_int());
        let response = self.authenticated_post(&url, &control_request).await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(HeatzyError::NotFound(format!("Device '{}' not found", device_id)));
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeatzyError::Api(format!("Failed to control device with status {}: {}", status, error_text)));
        }
        
        info!("Successfully set device mode");
        Ok(())
    }
    
    /// Helper to ensure we have a token
    fn ensure_authenticated(&self) -> Result<(), HeatzyError> {
        if self.token.is_none() {
            return Err(HeatzyError::NoToken);
        }
        Ok(())
    }
    
    /// Helper for authenticated GET requests
    async fn authenticated_get(&self, url: &str) -> Result<reqwest::Response, HeatzyError> {
        let token = self.token.as_ref().ok_or(HeatzyError::NoToken)?;
        
        trace!("GET {}", url);
        self.http_client
            .get(url)
            .header(USER_TOKEN_HEADER, token)
            .send()
            .await
            .map_err(Into::into)
    }
    
    /// Helper for authenticated POST requests
    async fn authenticated_post<T: serde::Serialize>(&self, url: &str, body: &T) -> Result<reqwest::Response, HeatzyError> {
        let token = self.token.as_ref().ok_or(HeatzyError::NoToken)?;
        
        trace!("POST {}", url);
        self.http_client
            .post(url)
            .header(USER_TOKEN_HEADER, token)
            .json(body)
            .send()
            .await
            .map_err(Into::into)
    }
}