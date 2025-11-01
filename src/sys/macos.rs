use objc2::declare_class;
use objc2::runtime::{Class, Object, Sel};
use objc2_core_location::{CLAuthorizationStatus, CLLocationManager, CLLocationManagerDelegate};
use objc2_core_wlan::{CWNetwork, CWSecurity, CWWiFiClient};
use objc2_foundation::{MainThreadBound, NSObject};

use crate::{Error, Result, Wifi};

/// Returns a list of WiFi hotspots in your area - macOS uses `objc2-core-wlan`.
pub fn scan() -> Result<Vec<Wifi>> {
    unsafe {
        let client = CWWiFiClient::sharedWiFiClient();
        let interface = client.interface();
        let scanned = match interface {
            Some(ref iface) => iface.scanForNetworksWithName_error(None),
            None => return Err(Error::ScanFailed("No WiFi interface found.".to_string())),
        };

        let manager = CLLocationManager::new();
        let delegate = LocationDelegate::new();
        manager.setDelegate(Some(&delegate));
        manager.requestWhenInUseAuthorization();

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
                        channel: network
                            .wlanChannel()
                            .map_or(String::new(), |c| c.channelNumber().to_string()),
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

declare_class!(
    struct LocationDelegate;

    unsafe impl ClassType for LocationDelegate {
        type Super = NSObject;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "LocationDelegate";
    }

    unsafe impl CLLocationManagerDelegate for LocationDelegate {
        #[method(locationManagerDidChangeAuthorization:)]
        fn location_manager_did_change_authorization(
            &self,
            _manager: &CLLocationManager,
            status: CLAuthorizationStatus,
        ) {
            println!("Location status changed: {}.", status);
        }
    }
);
