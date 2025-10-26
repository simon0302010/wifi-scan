use std::{thread::sleep, time::Duration};

use crate::{Error, Result, Wifi};

use neli_wifi::Socket as SocketN;
use netlink_rust::{Protocol, generic, Socket};
use nl80211_rs as nl80211;

/// Returns a list of WiFi hotspots in your area
pub(crate) fn scan() -> Result<Vec<Wifi>> {
    let socket = SocketN::connect();
    if let Ok(mut socket_conn) = socket {
        match socket_conn.get_interfaces_info() {
            Ok(interfaces) => {
                for interface in interfaces {
                    if let Some(index) = interface.index {
                        // trigger scan on interface
                        trigger_scan(index)?;

                        let mut results: Vec<Wifi> = Vec::new();
                        let bss_list = socket_conn.get_bss_info(index);
                        if let Ok(bss_list) = bss_list {for bss in bss_list {
                            if let Some(seen) = bss.seen_ms_ago {
                                if seen <= 3000 {
                                    results.push(Wifi {
                                        mac: match bss.bssid {
                                            Some(bytes) => convert_mac(bytes),
                                            None => String::new(),
                                        },
                                        ssid: String::new(),
                                        channel: match bss.frequency {
                                            Some(frequency) => get_channel(frequency),
                                            None => String::new()
                                        },
                                        signal_level: match bss.signal {
                                            Some(signal) => format!("{:.2}", signal as f32 / 100.0),
                                            None => String::new()
                                        },
                                        security: String::new(),
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