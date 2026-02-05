/// WiFi SoftAP setup for ESP32-C6
///
/// Creates a standalone WiFi hotspot. No internet needed.
/// Phones connect directly to the device WiFi network.
extern crate alloc;

use alloc::format;
use alloc::string::String;

use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, EspWifi};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_hal::modem::Modem;

use crate::config;

/// Initialize WiFi in SoftAP mode
///
/// Returns the EspWifi instance (must be kept alive for WiFi to remain active)
pub fn init_softap(
    modem: Modem,
    sysloop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
) -> Result<BlockingWifi<EspWifi<'static>>, Box<dyn std::error::Error>> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), nvs)?,
        sysloop,
    )?;

    let ap_config = AccessPointConfiguration {
        ssid: config::AP_SSID.try_into().unwrap(),
        password: config::AP_PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        max_connections: config::AP_MAX_CONNECTIONS,
        ..Default::default()
    };

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::AccessPoint(ap_config))?;

    wifi.start()?;

    log::info!(
        "WiFi SoftAP started: SSID='{}', IP={}",
        config::AP_SSID,
        config::AP_IP
    );

    Ok(wifi)
}

/// Generate WiFi QR code string for auto-connect
///
/// Format: WIFI:T:WPA;S:<SSID>;P:<password>;;
pub fn wifi_qr_string() -> String {
    format!(
        "WIFI:T:WPA;S:{};P:{};;",
        config::AP_SSID, config::AP_PASSWORD
    )
}

/// Generate web UI URL
pub fn web_url() -> String {
    format!("http://{}", config::AP_IP)
}

