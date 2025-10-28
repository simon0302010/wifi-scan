# wifi_scan

[![CI](https://github.com/simon0302010/wifi-scan/actions/workflows/ci.yml/badge.svg)](https://github.com/simon0302010/wifi-scan/actions/workflows/ci.yml)
[![Crates](https://img.shields.io/crates/v/wifi_scan.svg)](https://crates.io/crates/wifi_scan)
[![docs.rs](https://docs.rs/wifi_scan/badge.svg)](https://docs.rs/wifi_scan)
[![dependency status](https://deps.rs/repo/github/simon0302010/wifi-scan/status.svg)](https://deps.rs/repo/github/simon0302010/wifi-scan)

## Intro

This is a fork of [wifiscanner](https://github.com/booyaa/wifiscanner), a crate to list WiFi hotspots in your area.

Note: Only macOS versions up to Ventura (13) are supported.

Tests taken from Christian Kuster's [node-wifi-scanner](https://github.com/ancasicolica/node-wifi-scanner)

Full documentation can be found [here](https://docs.rs/wifi_scan).

## Usage

This crate is [on crates.io](https://crates.io/crates/wifi_scan) and can be
used by adding `wifi_scan` to the dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
wifi_scan = "0.6.*"
```

and this to your crate root:

```rust
extern crate wifi_scan;
```

## Example

```rust
use wifi_scan;
println!("{:?}", wifi_scan::scan());
```

Alternatively if you've cloned the Git repo, you can run the above example
using: `cargo run --example scan`.

## Changelog

- 0.6.0 - remove `iw` dependency for linux
- 0.5.1 - crates.io metadata update
- 0.5.0 - add window support (props to  @brianjaustin)
- 0.4.0 - replace iwlist with iw (props to @alopatindev)
- 0.3.6 - crates.io metadata update
- 0.3.5 - remove hardcoded path for iwlist (props to @alopatindev)
- 0.3.4 - initial stable release

## How to contribute

see [CONTRIBUTING.md](/CONTRIBUTING.md)

## Contributors

wifi_scan would not be possible without the following folks:

@alopatindev, @bizzu, @bash, @cristicbz, @lpmi-13, @brianjaustin

## Copyright

Copyright 2019 Mark Sta Ana.  
Forked and maintained by simon0302010.

Copyright 2025 simon0302010.

see [LICENSE](/LICENSE)
