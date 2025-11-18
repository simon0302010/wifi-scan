use std::{thread::sleep, time::Duration};

use crate::{Error, Result, Wifi};

use neli_wifi::Socket as SocketN;
use netlink_rust::{generic, Protocol, Socket};
use nl80211_rs::{
    self as nl80211,
    information_element::{AuthenticationKeyManagement, InformationElement},
};

/// Returns a list of WiFi hotspots in your area.
/// Open networks are recognised as having WPA2-PSK on Linux.
/// Uses `nl80211-rs` and `netlink-rust` crates on Linux.
/// On Linux, very frequent scan may produce unexpected results on some machines,
/// scanning requires root privileges and results can be up to 2500ms old.
pub(crate) fn scan() -> Result<Vec<Wifi>> {
    let mut socket_conn = SocketN::connect().map_err(|e| Error::SocketError(e.to_string()))?;

    let interfaces = socket_conn
        .get_interfaces_info()
        .map_err(|e| Error::InterfaceError(e.to_string()))?;

    if interfaces.is_empty() {
        return Err(Error::InterfaceError(
            "No WiFi adapters detected".to_string(),
        ));
    }

    let mut filtered_wifis: Vec<Wifi> = Vec::new();
    let mut all_wifis: Vec<Wifi> = Vec::new();

    let scan_result = std::panic::catch_unwind(trigger_scan);

    // sleep if at least one scan succeeded
    match scan_result {
        Ok(inner) => match inner {
            Ok(_) => sleep(Duration::from_millis(1500)),
            Err(e) => {
                if e.to_string().contains("Operation not permitted") {
                    return Err(Error::ScanFailed(
                        "Operation not permitted. Try running as root.".to_string(),
                    ));
                }
            }
        },
        Err(_) => println!("WARNING: Code to trigger WiFi scan panicked"),
    }

    for interface in &interfaces {
        if let Some(index) = interface.index {
            let mut results: Vec<Wifi> = Vec::new();
            let bss_list = socket_conn.get_bss_info(index);
            if let Ok(bss_list) = bss_list {
                for bss in bss_list {
                    if let Some(seen) = bss.seen_ms_ago {
                        if seen <= 2500 {
                            results.push(Wifi {
                                mac: match bss.bssid {
                                    Some(bytes) => convert_mac(bytes),
                                    None => String::new(),
                                },
                                ssid: match bss.information_elements.clone() {
                                    Some(ie_data) => get_ssid(ie_data),
                                    None => String::new(),
                                },
                                channel: match bss.frequency {
                                    Some(frequency) => get_channel(frequency),
                                    None => 0,
                                },
                                signal_level: match bss.signal {
                                    Some(signal) => {
                                        format!("{:.2}", signal as f32 / 100.0)
                                    }
                                    None => String::new(),
                                },
                                security: match bss.information_elements.clone() {
                                    Some(ie_data) => get_security(ie_data),
                                    None => String::new(),
                                },
                            });
                        }
                    }
                }
            }

            all_wifis.extend(results);
        }
    }
    for wifi in all_wifis {
        let exists = filtered_wifis.iter().any(|x| x.mac == wifi.mac);
        if !exists {
            filtered_wifis.push(wifi);
        }
    }
    Ok(filtered_wifis)
}

/// Returns Ok() if at least one scan succeeded
fn trigger_scan() -> Result<()> {
    let mut control_socket = match Socket::new(Protocol::Generic) {
        Ok(sock) => sock,
        Err(e) => return Err(Error::SocketError(e.to_string())),
    };

    let family = match generic::Family::from_name(&mut control_socket, "nl80211") {
        Ok(fam) => fam,
        Err(e) => return Err(Error::SocketError(e.to_string())),
    };

    let devices = match nl80211::get_wireless_interfaces(&mut control_socket, &family) {
        Ok(dev) => dev,
        Err(e) => return Err(Error::SocketError(e.to_string())),
    };

    let mut failed_count = 0;
    let mut one_succeeded = false;
    for dev in devices {
        match dev.trigger_scan(&mut control_socket) {
            Ok(_) => {
                one_succeeded = true;
                println!("Triggered scan on: {}", dev.interface_name)
            }
            Err(e) => {
                println!(
                    "WARNING: Failed to trigger scan on {}: {}",
                    dev.interface_name, e
                );
                failed_count += 1;
                if e.to_string().contains("not permitted") {
                    return Err(Error::ScanFailed(
                        "Operation not permitted. Try running as root.".to_string(),
                    ));
                }
            }
        }
    }

    if one_succeeded {
        Ok(())
    } else {
        Err(Error::ScanFailed(format!(
            "Triggering a network scan failed on {} devices.",
            failed_count
        )))
    }
}

fn convert_mac(bytes: Vec<u8>) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
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

fn get_ssid(ie_data: Vec<u8>) -> String {
    let ie_data: &[u8] = &ie_data;
    match InformationElement::parse_all(ie_data) {
        Ok(ies) => {
            for ie in ies {
                if let InformationElement::Ssid(ssid_ie) = ie {
                    return ssid_ie.ssid;
                }
            }
        }
        Err(_) => return String::new(),
    }

    String::new()
}

fn get_security(ie_data: Vec<u8>) -> String {
    let ie_data: &[u8] = &ie_data;
    match InformationElement::parse_all(ie_data) {
        Ok(ies) => {
            for ie in ies {
                if let InformationElement::RobustSecurityNetwork(sec_ie) = ie {
                    let mut securities: Vec<String> = Vec::new();

                    for akm in sec_ie.akms {
                        let security = match akm {
                            AuthenticationKeyManagement::PreSharedKey => "WPA2-Personal (PSK)",
                            AuthenticationKeyManagement::SimultaneousAuthenticationOfEquals => {
                                "WPA3-Personal (SAE)"
                            }
                            AuthenticationKeyManagement::PairwiseMasterKeySecurityAssociation => {
                                "WPA2-Enterprise (EAP)"
                            }
                            AuthenticationKeyManagement::PMKSASha256 => "WPA3-Enterprise (EAP-256)",
                            AuthenticationKeyManagement::FastTransitionPMKSA => {
                                "WPA2-Enterprise (EAP-FT)"
                            }
                            AuthenticationKeyManagement::PreSharedKeySha256 => {
                                "WPA3-Personal (PSK-256)"
                            }
                            AuthenticationKeyManagement::FastTransitionPreSharedKey => {
                                "WPA2-Personal (PSK-FT)"
                            }
                            AuthenticationKeyManagement::FastTransitionSAE => {
                                "WPA3-Personal (SAE-FT)"
                            }
                            AuthenticationKeyManagement::TunneledDirectLinkSetup => "TDLS",
                            _ => "",
                        };

                        if !security.is_empty() {
                            securities.push(security.to_string());
                        }
                    }

                    if securities.is_empty() {
                        securities.push("Unknown".to_string());
                    }

                    return securities.join(", ");
                }
            }
        }
        Err(_) => return String::new(),
    }

    String::new()
}
