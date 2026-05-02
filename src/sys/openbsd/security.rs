#![allow(unused)]

use crate::{WifiSecurity, sys::openbsd::lswifi::{ConstCharArray, ScanResult}};

const IEEE80211_WPA_PROTO_WPA1: u32 = 0x01;
const IEEE80211_WPA_PROTO_WPA2: u32 = 0x02;

const IEEE80211_WPA_CIPHER_NONE: u32 = 0x00;
const IEEE80211_WPA_CIPHER_USEGROUP: u32 = 0x01;
const IEEE80211_WPA_CIPHER_WEP40: u32 = 0x02;
const IEEE80211_WPA_CIPHER_TKIP: u32 = 0x04;
const IEEE80211_WPA_CIPHER_CCMP: u32 = 0x08;
const IEEE80211_WPA_CIPHER_WEP104: u32 = 0x10;
const IEEE80211_WPA_CIPHER_BIP: u32 = 0x20;

const IEEE80211_WPA_AKM_PSK: u32 = 0x01;
const IEEE80211_WPA_AKM_8021X: u32 = 0x02;
const IEEE80211_WPA_AKM_SHA256_PSK: u32 = 0x04;
const IEEE80211_WPA_AKM_SHA256_8021X: u32 = 0x08;
const IEEE80211_WPA_AKM_SAE: u32 = 0x10;

const IEEE80211_CAPINFO_PRIVACY: u32 = 0x0010;

impl ScanResult {
    pub fn get_security(&self) -> Vec<WifiSecurity> {
        let features = parse_features(self);
        let mut securities = Vec::new();

        if features.is_empty() {
            return vec![WifiSecurity::Open];
        }

        if features.contains(&"wpa2") {
            if features.contains(&"psk sha256") {
                securities.push(WifiSecurity::Wpa2PersonalPsk256);
            } else if features.contains(&"psk") {
                securities.push(WifiSecurity::Wpa2PersonalPsk);
            }
            if features.contains(&"802.1x sha256") {
                securities.push(WifiSecurity::Wpa2EnterpriseEap256);
            } else if features.contains(&"802.1x") {
                securities.push(WifiSecurity::Wpa2EnterpriseEap);
            }
        }

        if features.contains(&"wpa3") {
            securities.push(WifiSecurity::Wpa3PersonalSae);
            // OpenBSD cannot detect WPA3-Enterprise.
        }

        if features.contains(&"wpa1") {
            if features.contains(&"psk") {
                securities.push(WifiSecurity::WpaPersonal);
            } else if features.contains(&"802.1x") {
                securities.push(WifiSecurity::WpaEnterprise);
            }
        }

        if features.contains(&"wep") {
            securities.push(WifiSecurity::Wep);
        }

        securities
    }
}

fn parse_features(wifi: &ScanResult) -> Vec<&'static str> {
    let mut features = Vec::new();

    if wifi.nr_capinfo != 0 {
        if wifi.nr_capinfo & IEEE80211_CAPINFO_PRIVACY != 0 {
            if wifi.nr_rsnprotos != 0 {
                if wifi.nr_rsnprotos & IEEE80211_WPA_PROTO_WPA2 != 0 {
                    if wifi.nr_rsnakms & IEEE80211_WPA_AKM_SAE != 0 {
                        if wifi.nr_rsnakms == IEEE80211_WPA_AKM_SAE {
                            features.push("wpa3");
                        } else {
                            features.push("wpa3");
                            features.push("wpa2");
                        }
                    } else {
                        features.push("wpa2");
                    }
                }

                if wifi.nr_rsnprotos & IEEE80211_WPA_PROTO_WPA1 != 0 {
                    features.push("wpa1");
                }
            } else {
                features.push("wep");
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_8021X != 0 {
                features.push("802.1x");
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_SHA256_8021X != 0 {
                features.push("802.1x sha256");
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_PSK != 0 {
                features.push("psk");
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_SHA256_PSK != 0 {
                features.push("psk sha256");
            }
        }
    }

    features
}