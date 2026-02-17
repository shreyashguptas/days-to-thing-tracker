/// WiFi setup: SoftAP (provisioning) and Station (normal) modes
///
/// First boot: SoftAP mode for WiFi provisioning via web UI
/// Subsequent boots: Station mode joining user's home WiFi
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use esp_idf_hal::modem::Modem;

use esp_idf_svc::handle::RawHandle;

use crate::config;

/// Type alias for the WiFi handle that must be kept alive
pub type BlockingWifiHandle = BlockingWifi<EspWifi<'static>>;

/// Saved WiFi credentials from NVS
#[derive(Debug, Clone)]
pub struct WiFiCredentials {
    pub ssid: String,
    pub password: String,
}

/// Current WiFi operating mode
#[derive(Debug, Clone)]
pub enum WiFiMode {
    AccessPoint { ip: [u8; 4] },
    Station { ssid: String, ip: [u8; 4] },
}

impl WiFiMode {
    pub fn is_station(&self) -> bool {
        matches!(self, WiFiMode::Station { .. })
    }

    pub fn ip(&self) -> [u8; 4] {
        match self {
            WiFiMode::AccessPoint { ip } => *ip,
            WiFiMode::Station { ip, .. } => *ip,
        }
    }

    pub fn mode_str(&self) -> &str {
        match self {
            WiFiMode::AccessPoint { .. } => "ap",
            WiFiMode::Station { .. } => "sta",
        }
    }

    pub fn ssid(&self) -> Option<&str> {
        match self {
            WiFiMode::Station { ssid, .. } => Some(ssid.as_str()),
            _ => None,
        }
    }
}

/// Scanned WiFi network info
#[derive(Debug, Clone)]
pub struct ScannedNetwork {
    pub ssid: String,
    pub rssi: i8,
    pub auth: String,
}

/// Load WiFi credentials from NVS
pub fn load_wifi_creds(nvs_partition: &EspDefaultNvsPartition) -> Option<WiFiCredentials> {
    let nvs = EspNvs::new(nvs_partition.clone(), config::NVS_NAMESPACE, true).ok()?;

    let mut ssid_buf = [0u8; 64];
    let ssid = nvs.get_str(config::NVS_KEY_SSID, &mut ssid_buf).ok()??;
    let ssid = String::from(ssid);

    if ssid.is_empty() {
        return None;
    }

    let mut pass_buf = [0u8; 128];
    let password = nvs
        .get_str(config::NVS_KEY_PASSWORD, &mut pass_buf)
        .ok()?
        .unwrap_or("");
    let password = String::from(password);

    log::info!("Loaded WiFi credentials for SSID: {}", ssid);
    Some(WiFiCredentials { ssid, password })
}

/// Save WiFi credentials to NVS
pub fn save_wifi_creds(
    nvs_partition: &EspDefaultNvsPartition,
    ssid: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut nvs = EspNvs::new(nvs_partition.clone(), config::NVS_NAMESPACE, true)?;
    nvs.set_str(config::NVS_KEY_SSID, ssid)?;
    nvs.set_str(config::NVS_KEY_PASSWORD, password)?;
    log::info!("Saved WiFi credentials for SSID: {}", ssid);
    Ok(())
}

/// Clear WiFi credentials from NVS
pub fn clear_wifi_creds(
    nvs_partition: &EspDefaultNvsPartition,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut nvs = EspNvs::new(nvs_partition.clone(), config::NVS_NAMESPACE, true)?;
    let _ = nvs.remove(config::NVS_KEY_SSID);
    let _ = nvs.remove(config::NVS_KEY_PASSWORD);
    log::info!("Cleared WiFi credentials from NVS");
    Ok(())
}

/// Initialize WiFi in Station mode (connect to user's home WiFi)
pub fn init_station(
    modem: Modem,
    sysloop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
    creds: &WiFiCredentials,
) -> Result<(BlockingWifi<EspWifi<'static>>, [u8; 4]), Box<dyn std::error::Error>> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), nvs)?,
        sysloop,
    )?;

    let client_config = ClientConfiguration {
        ssid: creds.ssid.as_str().try_into().map_err(|_| "SSID too long")?,
        password: creds.password.as_str().try_into().map_err(|_| "Password too long")?,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    };

    wifi.set_configuration(&Configuration::Client(client_config))?;
    wifi.start()?;

    log::info!("WiFi STA started, connecting to '{}'...", creds.ssid);

    wifi.connect()?;

    // Wait for connection with timeout
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    let ip = ip_info.ip.octets();

    log::info!(
        "WiFi STA connected to '{}', IP: {}.{}.{}.{}",
        creds.ssid,
        ip[0], ip[1], ip[2], ip[3]
    );

    Ok((wifi, ip))
}

/// Initialize WiFi in SoftAP mode (for provisioning)
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

    // Use Mixed mode (AP+STA) so WiFi scanning works during provisioning
    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration::default(),
        ap_config,
    ))?;

    wifi.start()?;

    log::info!(
        "WiFi SoftAP started: SSID='{}', IP={}",
        config::AP_SSID,
        config::AP_IP
    );

    Ok(wifi)
}

/// Scan for available WiFi networks (must be called while WiFi is started)
pub fn scan_networks(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
) -> Vec<ScannedNetwork> {
    match wifi.scan() {
        Ok(aps) => {
            let mut networks: Vec<ScannedNetwork> = aps
                .iter()
                .filter(|ap| !ap.ssid.is_empty())
                .map(|ap| ScannedNetwork {
                    ssid: String::from(ap.ssid.as_str()),
                    rssi: ap.signal_strength as i8,
                    auth: match ap.auth_method.unwrap_or(AuthMethod::None) {
                        AuthMethod::None => String::from("open"),
                        AuthMethod::WEP => String::from("wep"),
                        AuthMethod::WPA => String::from("wpa"),
                        AuthMethod::WPA2Personal => String::from("wpa2"),
                        AuthMethod::WPA3Personal => String::from("wpa3"),
                        AuthMethod::WPA2WPA3Personal => String::from("wpa2/wpa3"),
                        _ => String::from("secured"),
                    },
                })
                .collect();

            // Sort by signal strength (strongest first)
            networks.sort_by(|a, b| b.rssi.cmp(&a.rssi));

            // Deduplicate by SSID (keep strongest signal)
            let mut seen = alloc::collections::BTreeSet::new();
            networks.retain(|n| seen.insert(n.ssid.clone()));

            networks
        }
        Err(e) => {
            log::error!("WiFi scan failed: {}", e);
            Vec::new()
        }
    }
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

/// Generate WiFi QR code string for auto-connect (AP mode)
pub fn wifi_qr_string() -> String {
    format!(
        "WIFI:T:WPA;S:{};P:{};;",
        config::AP_SSID, config::AP_PASSWORD
    )
}

/// Generate web UI URL from IP
pub fn web_url_from_ip(ip: [u8; 4]) -> String {
    format!("http://{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// Stop WiFi for power saving (call before entering light sleep)
pub fn stop_wifi(wifi: &mut BlockingWifiHandle) -> Result<(), Box<dyn std::error::Error>> {
    let _ = wifi.disconnect();
    wifi.stop()?;
    log::info!("WiFi stopped for power saving");
    Ok(())
}

/// Restart WiFi after waking from light sleep (STA mode)
pub fn restart_wifi(wifi: &mut BlockingWifiHandle) -> Result<[u8; 4], Box<dyn std::error::Error>> {
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    let ip = ip_info.ip.octets();
    log::info!("WiFi restarted, IP: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    Ok(ip)
}

