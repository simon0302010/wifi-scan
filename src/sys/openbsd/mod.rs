use crate::{Error, Result, Wifi, WlanScanner, sys::openbsd::lswifi::{ConstCharArray, NetworkList, ScanResult, free_networks, get_networks}};

mod lswifi;

pub struct ScanOpenBsd;

impl WlanScanner for ScanOpenBsd {
    fn scan(&mut self) -> Result<Vec<Wifi>> {
        unsafe {
            let networks_ptr = get_networks();
            if networks_ptr.is_null() {
                return Err(Error::ScanFailed("Unknown error occurred".to_string()));
            }
            let networks: Vec<ScanResult> = NetworkList(networks_ptr).into();

            // TODO: missing fields
            let result = networks.iter().map(|network| {
                Wifi {
                    mac: ConstCharArray(network.bssid).into(),
                    ssid: ConstCharArray(network.ssid).into(),
                    channel: 0,
                    signal_level: network.rssi,
                    security: Vec::new()
                }
            }).collect();

            free_networks(networks_ptr);

            Ok(result)
        }
    }
}