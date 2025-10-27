use std::{thread::sleep, time::Duration};

use crate::{Error, Result, Wifi};

use neli_wifi::Socket as SocketN;
use netlink_rust::{Protocol, generic, Socket};
use nl80211_rs::{self as nl80211, information_element::{AuthenticationKeyManagement, InformationElement}};

/// Returns a list of WiFi hotspots in your area. Open networks are recognised as having WPA2-PSK on Linux.
pub(crate) fn scan() -> Result<Vec<Wifi>> {
    let socket = SocketN::connect();
    if let Ok(mut socket_conn) = socket {
        match socket_conn.get_interfaces_info() {
            Ok(interfaces) => {
                for interface in interfaces {
                    if let Some(index) = interface.index {
                        // trigger scan on interface
                        if let Err(_) = trigger_scan(index) {
                            continue;
                        }

                        // just sleep a bit
                        sleep(Duration::from_millis(1500));

                        let mut results: Vec<Wifi> = Vec::new();
                        let bss_list = socket_conn.get_bss_info(index);
                        if let Ok(bss_list) = bss_list {for bss in bss_list {
                            if let Some(seen) = bss.seen_ms_ago {
                                if seen <= 3000 {
                                    results.push(Wifi {
                                        mac: match bss.bssid {
                                            Some(bytes) => convert_mac(bytes),
                                            None => String::new()
                                        },
                                        ssid: match bss.information_elements.clone() {
                                            Some(ie_data) => get_ssid(ie_data),
                                            None => String::new()
                                        },
                                        channel: match bss.frequency {
                                            Some(frequency) => get_channel(frequency),
                                            None => String::new()
                                        },
                                        signal_level: match bss.signal {
                                            Some(signal) => format!("{:.2}", signal as f32 / 100.0),
                                            None => String::new()
                                        },
                                        security: match bss.information_elements.clone() {
                                            Some(ie_data) => get_security(ie_data),
                                            None => String::new()
                                        }
                                    });
                                }
                            }
                        }}

                        // TODO: combine results of multiple interfaces
                        return Ok(results);
                    }
                }
                Ok(Vec::new())
            }
            Err(e) => {
                Err(Error::InterfaceError(e.to_string()))
            }
        }
    } else if let Err(e) = socket {
        Err(Error::SocketError(e.to_string()))
    } else {
        Err(Error::NoValue)
    }
}

fn trigger_scan(ifindex: i32) -> Result<()> {
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
    
    let device = devices.into_iter()
        .find(|dev| dev.interface_index == ifindex as u32)
        .ok_or_else(|| Error::InterfaceError(format!("Interface {} not found", ifindex)))?;
    
    println!("Triggering scan on interface: {}", device.interface_name);
    match device.trigger_scan(&mut control_socket) {
        Ok(_) => {
            println!("  Scan triggered successfully");
            return Ok(())
        },
        Err(e) => return Err(Error::SocketError(format!("Failed to trigger scan: {}", e))),
    }    
}

fn convert_mac(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(":")
}

fn get_channel(frequency: u32) -> String {
    if (2412..=2472).contains(&frequency) {
        ((frequency - 2407) / 5).to_string()
    } else if frequency == 2484 {
        "14".to_string() // special case (Japan)
    } else if (5180..=5895).contains(&frequency) {
        ((frequency - 5000) / 5).to_string()
    } else if (5955..=7115).contains(&frequency) {
        ((frequency - 5950) / 5).to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_ssid(ie_data: Vec<u8>) -> String {
    let ie_data: &[u8] = &ie_data;
    match InformationElement::parse_all(&ie_data) {
        Ok(ies) => {
            for ie in ies {
                if let InformationElement::Ssid(ssid_ie) = ie {
                    return ssid_ie.ssid;
                }
            }
        }
        Err(_) => return String::new()
    }

    return String::new();
}

fn get_security(ie_data: Vec<u8>) -> String {
    let ie_data: &[u8] = &ie_data;
    match InformationElement::parse_all(&ie_data) {
        Ok(ies) => {
            for ie in ies {
                if let InformationElement::RobustSecurityNetwork(sec_ie) = ie {
                    let mut securities: Vec<String> = Vec::new();
                    
                    for akm in sec_ie.akms {
                        let security = match akm {
                            AuthenticationKeyManagement::PairwiseMasterKeySecurityAssociation => "WPA2-Enterprise",
                            AuthenticationKeyManagement::PreSharedKey => "WPA2-PSK",
                            AuthenticationKeyManagement::FastTransitionPMKSA => "WPA2-Enterprise-FT",
                            AuthenticationKeyManagement::FastTransitionPreSharedKey => "WPA2-PSK-FT",
                            AuthenticationKeyManagement::FastTransitionSAE => "WPA3-SAE-FT",
                            AuthenticationKeyManagement::PMKSASha256 => "WPA2-Enterprise-SHA256",
                            AuthenticationKeyManagement::PreSharedKeySha256 => "WPA2-PSK-SHA256",
                            AuthenticationKeyManagement::SimultaneousAuthenticationOfEquals => "WPA3-SAE",
                            AuthenticationKeyManagement::TunneledDirectLinkSetup => "TDLS",
                            _ => ""
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
        Err(_) => return String::new()
    }

    return String::new();
}