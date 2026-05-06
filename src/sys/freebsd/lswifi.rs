use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
};

#[repr(C)]
#[derive(Clone)]
pub struct ScanResult {
    pub interface: *mut c_char,
    pub bssid: *mut c_char,
    pub ssid: *mut c_char,
    pub rssi: c_int,
    pub channel: c_int,
}

unsafe extern "C" {
    pub unsafe fn get_networks() -> *mut *mut ScanResult;
    pub unsafe fn free_networks(networks: *mut *mut ScanResult);
}

pub struct NetworkList(pub *mut *mut ScanResult);

impl From<NetworkList> for Vec<ScanResult> {
    fn from(value: NetworkList) -> Self {
        unsafe {
            let mut vec = Vec::new();
            let mut i = 0;
            while !(*value.0.add(i)).is_null() {
                vec.push((*(*value.0.add(i))).clone());
                i += 1;
            }
            vec
        }
    }
}

pub struct ConstCharArray(pub *const c_char);

impl From<ConstCharArray> for String {
    fn from(value: ConstCharArray) -> Self {
        if value.0.is_null() {
            return String::new();
        }

        let c_str: &CStr = unsafe { CStr::from_ptr(value.0) };
        c_str.to_string_lossy().into_owned()
    }
}
