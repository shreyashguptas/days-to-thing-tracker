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

use esp_idf_svc::handle::RawHandle;

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

/// Get the AP's actual IP address and configure DHCP to advertise it as DNS server.
/// This enables captive portal detection on phones.
/// Returns the IP as [u8; 4].
pub fn configure_captive_portal(wifi: &BlockingWifi<EspWifi<'static>>) -> [u8; 4] {
    let netif = wifi.wifi().ap_netif();
    let ip_info = netif.get_ip_info().unwrap();
    let octets = ip_info.ip.octets();
    let handle = netif.handle();

    unsafe {
        use esp_idf_svc::sys::*;

        // Stop DHCP server to modify settings
        esp_netif_dhcps_stop(handle);

        // Set our IP as the DNS server
        let mut dns: esp_netif_dns_info_t = core::mem::zeroed();
        dns.ip.u_addr.ip4.addr = u32::from_ne_bytes(octets);
        dns.ip.type_ = 0; // IPADDR_TYPE_V4

        esp_netif_set_dns_info(
            handle,
            esp_netif_dns_type_t_ESP_NETIF_DNS_MAIN,
            &mut dns,
        );

        // Tell DHCP server to offer our DNS to clients
        let mut val: u8 = 1;
        esp_netif_dhcps_option(
            handle,
            esp_netif_dhcp_option_mode_t_ESP_NETIF_OP_SET,
            esp_netif_dhcp_option_id_t_ESP_NETIF_DOMAIN_NAME_SERVER,
            &mut val as *mut u8 as *mut core::ffi::c_void,
            core::mem::size_of::<u8>() as u32,
        );

        // Restart DHCP server
        esp_netif_dhcps_start(handle);
    }

    log::info!(
        "Captive portal DNS configured: {}.{}.{}.{}",
        octets[0], octets[1], octets[2], octets[3]
    );

    octets
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

