use crate::{Result, Wifi, WlanScanner};

pub struct ScanOpenBsd;

impl WlanScanner for ScanOpenBsd {
    fn scan(&mut self) -> Result<Vec<Wifi>> {

    }
}