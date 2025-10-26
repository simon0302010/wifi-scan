use crate::{Error, Result, Wifi};

use neli_wifi::Socket;

/// Returns a list of WiFi hotspots in your area
pub(crate) fn scan() -> Result<Vec<Wifi>> {
    let socket = Socket::connect();
    if let Ok(mut socket_conn) = socket {
        match socket_conn.get_interfaces_info() {
            Ok(interfaces) => {
                for interface in interfaces {
                    if let Some(index) = interface.index {
                        let mut results: Vec<Wifi> = Vec::new();
                        let bss_list = socket_conn.get_bss_info(index);
                        if let Ok(bss_list) = bss_list {for bss in bss_list {
                            results.push(Wifi {
                                mac: match bss.bssid {
                                    Some(bytes) => convert_mac(bytes),
                                    None => String::new(),
                                },
                                ssid: String::new(),
                                channel: match bss.frequency {
                                    Some(frequency) => frequency.to_string(),
                                    None => String::new()
                                },
                                signal_level: match bss.signal {
                                    Some(signal) => format!("{:.2}", signal as f32 / 100.0),
                                    None => String::new()
                                },
                                security: String::new(),
                            });
                        }}

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

fn convert_mac(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(":")
}