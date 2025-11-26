use crate::{Error, Result, Wifi, WifiSecurity, WlanScanner};

use libwifi::{frame::components::RsnAkmSuite, parsers::parse_rsn_information};
use win32_wlan::query_system_interfaces;

pub struct ScanWindows;

impl WlanScanner for ScanWindows {
    /// Returns a list of WiFi hotspots in your area - Windows uses the `win32-wlan` crate.
    fn scan(&mut self) -> Result<Vec<Wifi>> {
        let interfaces = futures::executor::block_on(query_system_interfaces())
            .map_err(|e| Error::InterfaceError(e.to_string()))?;

        if let Some(interface) = interfaces.first() {
            let networks = interface
                .blocking_scan()
                .map_err(|e| Error::ScanFailed(e.to_string()))?;

            let wifi_list = networks
                .iter()
                .filter_map(|network| {
                    network.ssid().map(|ssid| Wifi {
                        mac: network.bss_id().to_string(),
                        ssid: ssid.to_string(),
                        channel: get_channel(network.ch_center_frequency() / 1000),
                        signal_level: network.rssi(),
                        security: get_security(network.information_frame()),
                    })
                })
                .collect();

            Ok(wifi_list)
        } else {
            Err(Error::InterfaceError(
                "No WiFi interfaces found".to_string(),
            ))
        }
    }
}

fn get_channel(frequency: u32) -> u32 {
    if (2412..=2472).contains(&frequency) {
        (frequency - 2407) / 5
    } else if frequency == 2484 {
        14 // japan
    } else if (5180..=5895).contains(&frequency) {
        (frequency - 5000) / 5
    } else if (5955..=7115).contains(&frequency) {
        (frequency - 5950) / 5
    } else {
        0
    }
}

fn get_security(ie_data: &[u8]) -> Vec<WifiSecurity> {
    let mut has_rsn = false;
    let mut has_wpa = false;
    let mut securities: Vec<WifiSecurity> = Vec::new();

    let mut i = 0;
    while i + 1 < ie_data.len() {
        let element_id = ie_data[i];
        let length = ie_data[i + 1] as usize;

        if i + 2 + length > ie_data.len() {
            break;
        }

        let element_data = &ie_data[i + 2..i + 2 + length];

        match element_id {
            48 => {
                has_rsn = true;
                if let Ok(rsn) = parse_rsn_information(element_data) {
                    let sec = format_security(&rsn.akm_suites);
                    if !sec.is_empty() {
                        securities.extend(sec.iter().cloned());
                    }
                }
            }
            221 => {
                if length >= 4 {
                    let oui = &element_data[0..3];
                    if oui == [0x00, 0x50, 0xF2] && element_data.get(3) == Some(&0x01) {
                        has_wpa = true;
                        securities.push(WifiSecurity::WpaPersonal);
                    }
                }
            }
            _ => {}
        }

        i += 2 + length;
    }

    if securities.is_empty() {
        if has_rsn || has_wpa {
            vec![WifiSecurity::WpaPersonal, WifiSecurity::Wpa2PersonalPsk]
        } else {
            vec![WifiSecurity::Open]
        }
    } else {
        securities
    }
}

fn format_security(akm_suites: &[RsnAkmSuite]) -> Vec<WifiSecurity> {
    let mut securities: Vec<WifiSecurity> = Vec::new();

    for akm_suite in akm_suites {
        let security = match akm_suite {
            RsnAkmSuite::PSK => WifiSecurity::Wpa2PersonalPsk,
            RsnAkmSuite::SAE => WifiSecurity::Wpa3PersonalSae,
            RsnAkmSuite::EAP => WifiSecurity::Wpa2EnterpriseEap,
            RsnAkmSuite::EAP256 => WifiSecurity::Wpa3EnterpriseEap256,
            RsnAkmSuite::EAPFT => WifiSecurity::Wpa2EnterpriseEapFt,
            RsnAkmSuite::PSK256 => WifiSecurity::Wpa3PersonalPsk256,
            RsnAkmSuite::PSKFT => WifiSecurity::Wpa2PersonalPskFt,
            RsnAkmSuite::SUITEBEAP256 => WifiSecurity::Wpa3EnterpriseSuiteBEap256,
            _ => WifiSecurity::Unknown,
        };

        if security != WifiSecurity::Unknown {
            securities.push(security);
        }
    }

    if securities.is_empty() {
        vec![WifiSecurity::Wpa2PersonalPsk]
    } else {
        securities
    }
}
