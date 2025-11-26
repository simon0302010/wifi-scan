// Copyright 2016 Mark Sta Ana, 2025 simon0302010.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)

//! A crate to list WiFi hotspots in your area.
//!
//! As of v0.5.x macOS, Windows and Linux are supported.
//! Use versions 0.6.* if you want a drop-in replacement for the original crate.
//!
//! # Usage
//!
//! This crate is on [crates.io](https://crates.io/crates/wifi_scan) and can be
//! used by adding `wifi_scan` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! wifi_scan = "0.7.*"
//! ```
//!
//! # Example
//!
//! ```no_run
//! use wifi_scan;
//! println!("{:?}", wifi_scan::scan());
//! ```
//!
//! Alternatively if you've cloned the the Git repo, you can run the above example
//! using: `cargo run --example scan`.

mod sys;

use std::fmt;

type Result<T> = std::result::Result<T, Error>;

/// Erros for wifi_scan
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InterfaceError(String),
    SocketError(String),
    ScanFailed(String),
}

/// Enum of WiFi Securities wifi_scan can output.
/// Not all implementations support all securities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiSecurity {
    Open,
    Wpa2PersonalPsk,
    Wpa3PersonalSae,
    Wpa2EnterpriseEap,
    Wpa3EnterpriseEap256,
    Wpa3EnterpriseSuiteBEap256,
    Wpa2EnterpriseEapFt,
    Wpa3PersonalPsk256,
    Wpa2PersonalPskFt,
    Wpa3PersonalSaeFt,
    Wep,
    WpaEnterprise,
    WpaPersonal,
    Personal,
    Enterprise,
    Tdls,
    Unknown,
    Other(String),
}

/// Wifi struct used to return information about wifi hotspots. Shows security on Linux since version 0.6.0.
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Wifi {
    /// MAC Address. May be empty on macOS.
    pub mac: String,
    /// Hotspot Name. May be empty on macOS.
    pub ssid: String,
    /// Channel the hotspot is on. Returns 0 if unknown.
    pub channel: u32,
    /// Wifi signal strength in dBm. Returns 0 if unknown.
    pub signal_level: i32,
    /// A list of all supported securities by the network
    pub security: Vec<WifiSecurity>,
}

/// Human readable signal strength
pub enum SignalStrength {
    Unknown,
    Weak,
    Fair,
    Good,
    Excellent,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SocketError(detail) => {
                write!(f, "Error while creating socket: {}", detail)
            }
            Error::InterfaceError(detail) => {
                write!(f, "Interface error: {}", detail)
            }
            Error::ScanFailed(detail) => {
                write!(f, "Scan Failed: {}", detail)
            }
        }
    }
}

impl fmt::Display for WifiSecurity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WifiSecurity::Enterprise => write!(f, "Enterprise"),
            WifiSecurity::Open => write!(f, "Open"),
            WifiSecurity::Other(sec) => write!(f, "{}", sec),
            WifiSecurity::Personal => write!(f, "Personal"),
            WifiSecurity::Tdls => write!(f, "TLDS"),
            WifiSecurity::Unknown => write!(f, "Unknown"),
            WifiSecurity::Wep => write!(f, "WEP"),
            WifiSecurity::Wpa2EnterpriseEap => write!(f, "WPA2-Enterprise (EAP)"),
            WifiSecurity::Wpa2EnterpriseEapFt => write!(f, "WPA2-Enterprise (EAP-FT)"),
            WifiSecurity::Wpa2PersonalPsk => write!(f, "WPA2-Personal (PSK)"),
            WifiSecurity::Wpa2PersonalPskFt => write!(f, "WPA2-Personal (PSK-FT)"),
            WifiSecurity::Wpa3EnterpriseEap256 => write!(f, "WPA3-Enterprise (EAP-256)"),
            WifiSecurity::Wpa3EnterpriseSuiteBEap256 => {
                write!(f, "WPA3-Enterprise (Suite B EAP-256)")
            }
            WifiSecurity::Wpa3PersonalPsk256 => write!(f, "WPA3-Personal (PSK-256)"),
            WifiSecurity::Wpa3PersonalSae => write!(f, "WPA3-Personal (SAE)"),
            WifiSecurity::Wpa3PersonalSaeFt => write!(f, "WPA3-Personal (SAE-FT)"),
            WifiSecurity::WpaEnterprise => write!(f, "WPA-Enterprise"),
            WifiSecurity::WpaPersonal => write!(f, "WPA-Personal"),
        }
    }
}

impl fmt::Display for Wifi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[MAC: {} | SSID: {} | Channel: {} | RSSI: {} dBm | Security: {}",
            self.mac,
            self.ssid,
            self.channel,
            self.signal_level,
            self.security
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Wifi {
    /// Returns `true` if the network is open
    pub fn is_open(&self) -> bool {
        self.security.len() == 1 && self.security[0] == WifiSecurity::Open
    }

    /// Returns `true` if the network supports WPA3
    pub fn is_wpa3(&self) -> bool {
        self.security.iter().any(|s| {
            matches!(
                s,
                WifiSecurity::Wpa3EnterpriseEap256
                    | WifiSecurity::Wpa3EnterpriseSuiteBEap256
                    | WifiSecurity::Wpa3PersonalPsk256
                    | WifiSecurity::Wpa3PersonalSae
                    | WifiSecurity::Wpa3PersonalSaeFt
            )
        })
    }

    /// Returns `true` if the network supports WPA2
    pub fn is_wpa2(&self) -> bool {
        self.security.iter().any(|s| {
            matches!(
                s,
                WifiSecurity::Wpa2EnterpriseEap
                    | WifiSecurity::Wpa2EnterpriseEapFt
                    | WifiSecurity::Wpa2PersonalPsk
                    | WifiSecurity::Wpa2PersonalPskFt
            )
        })
    }

    /// Returns `true` if the network is an enterprise network
    pub fn is_enterprise(&self) -> bool {
        self.security.iter().any(|s| {
            matches!(
                s,
                WifiSecurity::Enterprise
                    | WifiSecurity::WpaEnterprise
                    | WifiSecurity::Wpa2EnterpriseEap
                    | WifiSecurity::Wpa2EnterpriseEapFt
                    | WifiSecurity::Wpa3EnterpriseEap256
                    | WifiSecurity::Wpa3EnterpriseSuiteBEap256
            )
        })
    }

    /// Returns `true` if the wifi is a personal network
    pub fn is_personal(&self) -> bool {
        self.security.iter().any(|s| {
            matches!(
                s,
                WifiSecurity::Personal
                    | WifiSecurity::WpaPersonal
                    | WifiSecurity::Wpa2PersonalPsk
                    | WifiSecurity::Wpa2PersonalPskFt
                    | WifiSecurity::Wpa3PersonalPsk256
                    | WifiSecurity::Wpa3PersonalSae
                    | WifiSecurity::Wpa3PersonalSaeFt
            )
        })
    }

    /// Returns signal strength as a categorial value
    pub fn readable_signal(&self) -> SignalStrength {
        match self.signal_level {
            0 => SignalStrength::Unknown,
            -50..=0 => SignalStrength::Excellent,
            -70..=-61 => SignalStrength::Good,
            -80..=-71 => SignalStrength::Fair,
            _ => SignalStrength::Weak,
        }
    }

    /// Returns `true` if the network is hidden
    pub fn is_hidden(&self) -> bool {
        self.ssid.is_empty()
    }

    /// Returns WiFi frequency in MHz
    pub fn get_frequency(&self) -> u32 {
        match self.channel {
            1..=13 => 2407 + self.channel * 5,          // 2.4 GHz
            14 => 2484,                                 // 2.4 GHz (Japan)
            36..=165 => 5000 + self.channel * 5,        // 5 GHz
            167..=233 => 5950 + (self.channel - 1) * 5, // 6 GHz
            _ => 0,                                     // Invalid
        }
    }
}

impl std::error::Error for Error {}

pub trait WlanScanner {
    fn scan(&mut self) -> Result<Vec<Wifi>>;
}

/// Returns a list of WiFi hotspots in your area.
/// Uses `corewlan` on macOS and `win32-wlan` on Windows.
/// `nl80211-rs` and `netlink-rust` crates are being used on machines running Linux.
///
/// Example:
///
/// ```rust,no_run
/// use wifi_scan;
/// println!("{:?}", wifi_scan::scan());
/// ```
pub fn scan() -> Result<Vec<Wifi>> {
    #[cfg(target_os = "macos")]
    {
        let mut scanner = sys::macos::ScanMac;
        scanner.scan()
    }

    #[cfg(target_os = "linux")]
    {
        let mut scanner = sys::linux::ScanLinux;
        scanner.scan()
    }

    #[cfg(target_os = "windows")]
    {
        let mut scanner = sys::windows::ScanWindows;
        scanner.scan()
    }
}
