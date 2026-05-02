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
            (CWSecurity::None, vec![WifiSecurity::Open]),
            (CWSecurity::WEP, vec![WifiSecurity::Wep]),
            (CWSecurity::DynamicWEP, vec![WifiSecurity::Wep]),
            (CWSecurity::WpaPersonal, vec![WifiSecurity::WpaPersonalPsk]),
            (
                CWSecurity::WPA2Personal,
                vec![WifiSecurity::Wpa2PersonalPsk],
            ),
            (
                CWSecurity::WPA3Personal,
                vec![WifiSecurity::Wpa3PersonalSae],
            ),
            (
                CWSecurity::WpaPersonalMixed,
                vec![WifiSecurity::WpaPersonalPsk, WifiSecurity::Wpa2PersonalPsk],
            ),
            (
                CWSecurity::WPA3Transition,
                vec![WifiSecurity::Wpa3PersonalSae, WifiSecurity::Wpa2PersonalPsk],
            ),
            (
                CWSecurity::WpaEnterprise,
                vec![WifiSecurity::WpaEnterpriseEap],
            ),
            (
                CWSecurity::WPA2Enterprise,
                vec![WifiSecurity::Wpa2EnterpriseEap],
            ),
            (
                CWSecurity::WPA3Enterprise,
                vec![WifiSecurity::Wpa3EnterpriseEap],
            ),
            (
                CWSecurity::WpaEnterpriseMixed,
                vec![WifiSecurity::Wpa2EnterpriseEap],
            ),
            (CWSecurity::Enterprise, vec![WifiSecurity::Unknown]),
            (CWSecurity::Personal, vec![WifiSecurity::Unknown]),
            (CWSecurity::Unknown, vec![WifiSecurity::Unknown]),
        ];

        let mut securities: Vec<WifiSecurity> = Vec::new();

        for (security, security_enums) in &securities_dict {
            if network.supportsSecurity(security.clone()) {
                securities.extend(security_enums.clone());
            }
        }

        if securities.is_empty() {
            securities.push(WifiSecurity::Unknown);
        }

        securities
    }
}
