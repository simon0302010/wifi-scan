use objc2_core_wlan::{CWNetwork, CWSecurity, CWWiFiClient};

use crate::{Error, Result, Wifi};

/// Returns a list of WiFi hotspots in your area - macOS uses `objc2-core-wlan`.
/// Location Access must be granted to the program for it to display SSIDs and BSSIDs.
pub fn scan() -> Result<Vec<Wifi>> {
    unsafe {
        let client = CWWiFiClient::sharedWiFiClient();
        let interface = client.interface();
        let scanned = match interface {
            Some(ref iface) => iface.scanForNetworksWithName_error(None),
            None => return Err(Error::ScanFailed("No WiFi interface found.".to_string())),
        };

        let mut results: Vec<Wifi> = Vec::new();

        match scanned {
            Ok(networks) => {
                let networks_array = networks.allObjects();
                for network in networks_array.iter() {
                    results.push(Wifi {
                        mac: match network.bssid() {
                            Some(bssid) => bssid.to_string(),
                            None => String::new(),
                        },
                        ssid: network.ssid().map_or(String::new(), |s| s.to_string()),
                        channel: network.wlanChannel().map_or(0u32, |c| {
                            let ch = c.channelNumber();
                            if ch > 0 {
                                ch as u32
                            } else {
                                0u32
                            }
                        }),
                        signal_level: format!("{:.2}", network.rssiValue()),
                        security: get_security(&*network),
                    });
                }
                Ok(results)
            }
            Err(_) => Err(Error::ScanFailed("Scan failed.".to_string())),
        }
    }
}

fn get_security(network: &CWNetwork) -> String {
    unsafe {
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
}
