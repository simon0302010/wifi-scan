use crate::{
    sys::freebsd::lswifi::{free_networks, get_networks, ConstCharArray, NetworkList, ScanResult},
    Error, Result, Wifi, WifiSecurity, WlanScanner,
};

mod lswifi;

pub struct ScanFreeBsd;

impl WlanScanner for ScanFreeBsd {
    fn scan(&mut self) -> Result<Vec<Wifi>> {
        unsafe {
            let networks_ptr = get_networks();
            if networks_ptr.is_null() {
                let errno = std::io::Error::last_os_error();
                if errno.raw_os_error() == Some(libc::ENXIO) {
                    return Err(Error::InterfaceError("No interfaces found".to_string()));
                } else {
                    return Err(Error::ScanFailed(format!("{}", errno)));
                }
            }
            let networks: Vec<ScanResult> = NetworkList(networks_ptr).into();

            // TODO: missing fields
            let result = networks
                .iter()
                .map(|network| Wifi {
                    mac: ConstCharArray(network.bssid).into(),
                    ssid: ConstCharArray(network.ssid).into(),
                    channel: network.channel as u32,
                    signal_level: network.rssi,
                    security: vec![WifiSecurity::Unknown], // TODO: populate
                })
                .collect();

            free_networks(networks_ptr);

            Ok(result)
        }
    }
}
