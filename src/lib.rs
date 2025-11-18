// Copyright 2016 Mark Sta Ana, 2025 simon0302010.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)

//! A crate to list WiFi hotspots in your area.
//!
//! As of v0.5.x macOS, Windows and Linux are supported.
//!
//! # Usage
//!
//! This crate is on [crates.io](https://crates.io/crates/wifi_scan) and can be
//! used by adding `wifi_scan` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! wifi_scan = "0.6.*"
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

/// Wifi struct used to return information about wifi hotspots. Shows security on Linux since version 0.6.0.
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Wifi {
    /// mac address
    pub mac: String,
    /// hotspot name
    pub ssid: String,
    /// channel the hotspot is on
    pub channel: String,
    /// wifi signal strength in dBm
    pub signal_level: String,
    /// wifi security (e.g. WPA2-PSK)
    pub security: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SocketError(detail) => {
                write!(f, "Error while creating socket: {}", detail)
            }
            Error::InterfaceError(detail) => { // a
                write!(f, "Interface error: {}", detail)
            }
            Error::ScanFailed(detail) => { // a
                write!(f, "Scan Failed: {}", detail)
            }
        }
    }
}

impl std::error::Error for Error {}

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
    crate::sys::scan()
}
