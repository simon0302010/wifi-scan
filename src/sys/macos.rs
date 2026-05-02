use objc2_core_wlan::{CWNetwork, CWSecurity, CWWiFiClient};

use crate::{Error, Result, Wifi, WifiSecurity, WlanScanner};

pub struct ScanMac;

impl WlanScanner for ScanMac {
    /// Returns a list of WiFi hotspots in your area - macOS uses `objc2-core-wlan`.
    /// Location Access must be granted to the program for it to display SSIDs and BSSIDs.
    fn scan(&mut self) -> Result<Vec<Wifi>> {
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
                            signal_level: network.rssiValue() as i32,
                            security: get_security(&*network),
                        });
                    }
                    Ok(results)
                }
                Err(_) => Err(Error::ScanFailed("Scan failed.".to_string())),
            }
        }
    }
}

fn get_security(network: &CWNetwork) -> Vec<WifiSecurity> {
    unsafe {
        let securities_dict = vec![
            (CWSecurity::None, WifiSecurity::Open),
            (CWSecurity::WEP, WifiSecurity::Wep),
            (CWSecurity::DynamicWEP, WifiSecurity::Wep),

            (CWSecurity::WPAPersonal, WifiSecurity::WpaPersonal),
            (CWSecurity::WPA2Personal, WifiSecurity::Wpa2PersonalPsk),
            (CWSecurity::WPA3Personal, WifiSecurity::Wpa3PersonalSae),

            (CWSecurity::WPAPersonalMixed, WifiSecurity::Wpa2PersonalPsk), // usually wpa1/wpa2
            (CWSecurity::WPA3Transition, WifiSecurity::Wpa3PersonalSae),   // wpa2/wpa3 compatibility

            (CWSecurity::WPAEnterprise, WifiSecurity::WpaEnterprise),
            (CWSecurity::WPA2Enterprise, WifiSecurity::Wpa2EnterpriseEap),
            (CWSecurity::WPA3Enterprise, WifiSecurity::Wpa3EnterpriseEap256),
            (CWSecurity::WPAEnterpriseMixed, WifiSecurity::Wpa2EnterpriseEap),

            (CWSecurity::Enterprise, WifiSecurity::Enterprise),
            (CWSecurity::Personal, WifiSecurity::Personal),
            (CWSecurity::Unknown, WifiSecurity::Unknown),
        ];

        let mut securities: Vec<WifiSecurity> = Vec::new();

        for (security, security_enum) in &securities_dict {
            if network.supportsSecurity(security.clone()) {
                securities.push(security_enum.clone());
            }
        }

        if securities.is_empty() {
            securities.push(WifiSecurity::Unknown);
        }

        securities
    }
}
