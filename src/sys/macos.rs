use corewlan_sys::{self, CWNetwork, CWSecurity, CWWiFiClient};

use crate::{Error, Result, Wifi};

/// Returns a list of WiFi hotspots in your area - macOS uses `corewlan-sys`.
pub fn scan() -> Result<Vec<Wifi>> {
    let client = CWWiFiClient::sharedWiFiClient();
    let interface = client.interface();
    let scanned = interface.scanForNetworksWithName(None);

    let mut results: Vec<Wifi> = Vec::new();

    match scanned {
        Ok(networks) => {
            for network in networks {
                results.push(Wifi {
                    mac: match network.bssid() {
                        Some(bssid) => bssid,
                        None => String::new(),
                    },
                    ssid: network.ssid(),
                    channel: network.wlanChannel().number.to_string(),
                    signal_level: network.rssiValue().to_string(),
                    security: get_security(network),
                });
            }
            Ok(results)
        }
        Err(_) => Err(Error::ScanFailed("Scan failed.".to_string())),
    }
}

fn get_security(network: CWNetwork) -> String {
    let securities_dict = vec![
        (CWSecurity::None, "Open"),
        (CWSecurity::DynamicWEP, "Dynamic-WEP"),
        (CWSecurity::Enterprise, "Enterprise"),
        (CWSecurity::Personal, "Personal"),
        (CWSecurity::Unknown, "Unknown"),
        (CWSecurity::WEP, "WEP"),
        (CWSecurity::WPA2Enterprise, "WPA2-Enterprise"),
        (CWSecurity::WPA2Personal, "WPA2-Personal"),
        (CWSecurity::WPA3Enterprise, "WPA3-Enterprise"),
        (CWSecurity::WPA3Personal, "WPA3-Personal"),
        (CWSecurity::WPA3Transition, "WPA3-Transition"),
        (CWSecurity::WPAEnterprise, "WPA-Enterprise"),
        (CWSecurity::WPAEnterpriseMixed, "WPA-Enterprise-Mixed"),
        (CWSecurity::WPAPersonal, "WPA-Personal"),
        (CWSecurity::WPAPersonalMixed, "WPA-Personal-Mixed"),
    ];

    let mut securities: Vec<String> = Vec::new();

    for (security, security_str) in &securities_dict {
        if network.supportsSecurity(security.clone()) {
            securities.push(security_str.to_string());
        }
    }

    if securities.is_empty() {
        securities.push("Unknown".to_string());
    }

    securities.join(", ")
}
