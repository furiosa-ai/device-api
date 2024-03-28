use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::device::{DeviceCtrl, DeviceInner, DeviceMgmt, DevicePerf};
use crate::error::DeviceResult;
use crate::sysfs::npu_mgmt;
use crate::Arch;
use crate::ClockFrequency;
use crate::DeviceError;

#[derive(Clone)]
pub struct RenegadeInner {
    device_index: u8,
    sysfs: PathBuf,
    mgmt_root: PathBuf,
    // TODO: cache static results
}

impl RenegadeInner {
    pub fn new(device_index: u8, sysfs: PathBuf) -> Self {
        let mgmt_root = sysfs.join(format!(
            "class/renegade_mgmt/renegade!npu{device_index}mgmt"
        ));
        RenegadeInner {
            device_index,
            sysfs,
            mgmt_root,
        }
    }

    fn read_mgmt_to_string<P: AsRef<Path>>(&self, file: P) -> DeviceResult<String> {
        let path = self.mgmt_root.join(file);
        let value = fs::read_to_string(path)?;
        Ok(value.trim_end().to_string())
    }

    #[allow(dead_code)]
    fn write_ctrl_file<P: AsRef<Path>>(&self, file: P, contents: &str) -> DeviceResult<()> {
        let path = self.mgmt_root.join(file);
        std::fs::write(path, contents)?;
        Ok(())
    }
}

impl DeviceInner for RenegadeInner {}

impl DeviceMgmt for RenegadeInner {
    fn sysfs(&self) -> &PathBuf {
        &self.sysfs
    }

    fn device_index(&self) -> u8 {
        self.device_index
    }

    #[inline]
    fn arch(&self) -> Arch {
        Arch::Renegade
    }

    fn alive(&self) -> DeviceResult<bool> {
        self.read_mgmt_to_string(npu_mgmt::file::DEVICE_STATE)
            .map(|v| v == "good")
    }

    fn atr_error(&self) -> DeviceResult<HashMap<String, u32>> {
        self.read_mgmt_to_string(npu_mgmt::file::ATR_ERROR)
            .map(npu_mgmt::build_atr_error_map)
    }

    fn busname(&self) -> DeviceResult<String> {
        self.read_mgmt_to_string(npu_mgmt::file::BUS_NAME)
    }

    fn pci_dev(&self) -> DeviceResult<String> {
        self.read_mgmt_to_string(npu_mgmt::file::DEV)
    }

    fn device_sn(&self) -> DeviceResult<String> {
        self.read_mgmt_to_string(npu_mgmt::file::DEVICE_SN)
    }

    fn device_uuid(&self) -> DeviceResult<String> {
        self.read_mgmt_to_string(npu_mgmt::file::DEVICE_UUID)
    }

    fn firmware_version(&self) -> DeviceResult<String> {
        self.read_mgmt_to_string(npu_mgmt::file::FW_VERSION)
    }

    fn driver_version(&self) -> DeviceResult<String> {
        self.read_mgmt_to_string(npu_mgmt::file::VERSION)
    }

    fn heartbeat(&self) -> DeviceResult<u32> {
        self.read_mgmt_to_string(npu_mgmt::file::HEARTBEAT)
            .and_then(|str| {
                str.parse::<u32>().map_err(|_| {
                    DeviceError::unexpected_value(format!("Bad heartbeat value: {str}"))
                })
            })
    }

    fn clock_frequency(&self) -> DeviceResult<Vec<ClockFrequency>> {
        self.read_mgmt_to_string(npu_mgmt::file::NPU_CLOCKS)
            .map(|str| str.lines().flat_map(ClockFrequency::try_from).collect())
    }
}

impl DeviceCtrl for RenegadeInner {
    fn ctrl_device_led(&self, _led: (bool, bool, bool)) -> DeviceResult<()> {
        // XXX: must use DEVICE_LEDS file, not DEVICE_LED. Currently this it is not implemented on
        // the driver side.
        unimplemented!()
    }

    fn ctrl_ne_dtm_policy(&self, _policy: npu_mgmt::DtmPolicy) -> DeviceResult<()> {
        unimplemented!()
    }

    fn ctrl_performance_level(&self, _level: npu_mgmt::PerfLevel) -> DeviceResult<()> {
        unimplemented!()
    }

    fn ctrl_performance_mode(&self, _mode: npu_mgmt::PerfMode) -> DeviceResult<()> {
        unimplemented!()
    }
}

impl DevicePerf for RenegadeInner {
    fn get_performance_counter(
        &self,
        _file: &crate::DeviceFile,
    ) -> DeviceResult<crate::perf_regs::PerformanceCounter> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renegade_inner_functionality() -> eyre::Result<()> {
        let device = RenegadeInner::new(0, PathBuf::from("../test_data/test-1/sys"));

        assert_eq!(device.device_index(), 0);
        assert_eq!(device.arch(), Arch::Renegade);
        assert!(device.alive()?);
        assert_eq!(device.atr_error()?.len(), 9);
        assert_eq!(device.busname()?, "0000:00:03.0");
        assert_eq!(device.pci_dev()?, "235:0");
        assert_eq!(device.device_sn()?, "");
        assert_eq!(
            device.device_uuid()?,
            "82540B87-1055-48C6-AAB1-C4CC84672C71"
        );
        assert_eq!(device.firmware_version()?, "");
        assert_eq!(device.driver_version()?, "1.0.0, abcdefg");
        assert_eq!(device.heartbeat()?, 0);

        Ok(())
    }
}
