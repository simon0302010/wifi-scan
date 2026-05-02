#![allow(unused)]

use crate::sys::openbsd::lswifi::{ConstCharArray, ScanResult};

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

pub fn print_security(wifi: ScanResult) {
    let ssid: String = ConstCharArray(wifi.ssid).into();
    print!("{}:", ssid);

    if wifi.nr_capinfo != 0 {
        if wifi.nr_capinfo & IEEE80211_CAPINFO_PRIVACY != 0 {
            if wifi.nr_rsnprotos != 0 {
                if wifi.nr_rsnprotos & IEEE80211_WPA_PROTO_WPA2 != 0 {
                    if wifi.nr_rsnakms & IEEE80211_WPA_AKM_SAE != 0 {
                        if wifi.nr_rsnakms == IEEE80211_WPA_AKM_SAE {
                            print!(" [wpa3]");
                        } else {
                            print!(" [wpa3] [wpa2]");
                        }
                    } else {
                        print!(" [wpa2]");
                    }
                }

                if wifi.nr_rsnprotos & IEEE80211_WPA_PROTO_WPA1 != 0 {
                    print!(" [wpa1]");
                }
            } else {
                print!(" [wep]")
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_8021X != 0 {
                print!(" [802.1x]")
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_SHA256_8021X != 0 {
                print!(" [802.1x sha256]")
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_PSK != 0 {
                print!(" [psk]")
            }

            if wifi.nr_rsnakms & IEEE80211_WPA_AKM_SHA256_PSK != 0 {
                print!(" [psk sha256]")
            }
        }
    }
}