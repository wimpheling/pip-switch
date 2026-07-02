use std::ffi::CString;

use crate::Result;

pub const MSI_VENDOR_ID: u16 = 0x1462;
pub const MSI_PRODUCT_ID: u16 = 0x3fa4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidDeviceInfo {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial: Option<String>,
}

impl HidDeviceInfo {
    pub fn is_supported_msi_monitor(&self) -> bool {
        self.vendor_id == MSI_VENDOR_ID
            && self.product_id == MSI_PRODUCT_ID
            && self
                .product
                .as_deref()
                .map(|product| product.contains("MSI Gaming Controller"))
                .unwrap_or(true)
    }
}

pub trait HidConnection {
    fn write(&mut self, bytes: &[u8]) -> Result<usize>;
    fn read_timeout(&mut self, bytes: &mut [u8], timeout_ms: i32) -> Result<usize>;
}

pub trait HidTransport {
    fn devices(&self) -> Result<Vec<HidDeviceInfo>>;
    fn open_path(&self, path: &str) -> Result<Box<dyn HidConnection>>;
}

pub struct HidapiTransport {
    api: hidapi::HidApi,
}

impl HidapiTransport {
    pub fn new() -> Result<Self> {
        Ok(Self {
            api: hidapi::HidApi::new()?,
        })
    }
}

impl HidTransport for HidapiTransport {
    fn devices(&self) -> Result<Vec<HidDeviceInfo>> {
        Ok(self
            .api
            .device_list()
            .map(|device| HidDeviceInfo {
                path: device.path().to_string_lossy().into_owned(),
                vendor_id: device.vendor_id(),
                product_id: device.product_id(),
                manufacturer: device.manufacturer_string().map(ToOwned::to_owned),
                product: device.product_string().map(ToOwned::to_owned),
                serial: device.serial_number().map(ToOwned::to_owned),
            })
            .collect())
    }

    fn open_path(&self, path: &str) -> Result<Box<dyn HidConnection>> {
        let path = CString::new(path).map_err(|_| {
            crate::Error::Message("HID path contained an unexpected NUL byte".to_string())
        })?;
        Ok(Box::new(self.api.open_path(path.as_c_str())?))
    }
}

impl HidConnection for hidapi::HidDevice {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        Ok(hidapi::HidDevice::write(self, bytes)?)
    }

    fn read_timeout(&mut self, bytes: &mut [u8], timeout_ms: i32) -> Result<usize> {
        Ok(hidapi::HidDevice::read_timeout(self, bytes, timeout_ms)?)
    }
}
