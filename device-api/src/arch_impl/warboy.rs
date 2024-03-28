use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::device::{DeviceCtrl, DeviceInner, DeviceMgmt, DevicePerf};
use crate::error::DeviceResult;
use crate::perf_regs::PerformanceCounter;
use crate::sysfs::npu_mgmt;
use crate::{Arch, ClockFrequency, DeviceError, DeviceFile};

pub struct WarboyInner {
    device_index: u8,
    sysfs: PathBuf,
    mgmt_root: PathBuf,
    // TODO: cache static results
}

impl WarboyInner {
    pub fn new(device_index: u8, sysfs: PathBuf) -> Self {
        let mgmt_root = sysfs.join(format!("/class/npu_mgmt/npu{device_index}_mgmt/"));
        WarboyInner {
            device_index,
            sysfs,
            mgmt_root,
        }
    }

    fn read_mgmt_to_string<P: AsRef<Path>>(&self, file: P) -> DeviceResult<String> {
        let path = self.mgmt_root.join(file);
        let value = fs::read_to_string(path)?;
        Ok(value)
    }

    fn write_ctrl_file<P: AsRef<Path>>(&self, file: P, contents: &str) -> DeviceResult<()> {
        let path = self.mgmt_root.join(file);
        std::fs::write(path, contents)?;
        Ok(())
    }
}
impl DeviceInner for WarboyInner {}

impl DeviceMgmt for WarboyInner {
    fn sysfs(&self) -> &PathBuf {
        &self.sysfs
    }

    fn device_index(&self) -> u8 {
        self.device_index
    }

    #[inline]
    fn arch(&self) -> Arch {
        // TODO(n0gu): determine arch based on soc_rev
        Arch::WarboyB0
    }

    fn alive(&self) -> DeviceResult<bool> {
        self.read_mgmt_to_string(npu_mgmt::file::ALIVE)
            .and_then(|v| {
                npu_mgmt::parse_zero_or_one_to_bool(&v).ok_or_else(|| {
                    DeviceError::unexpected_value(format!(
                        "Bad alive value: {v} (only 0 or 1 expected)"
                    ))
                })
            })
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
        self.read_mgmt_to_string(npu_mgmt::file::NE_CLK_FREQ_INFO)
            .map(|str| str.lines().flat_map(ClockFrequency::try_from).collect())
    }
}

impl DeviceCtrl for WarboyInner {
    fn ctrl_device_led(&self, led: (bool, bool, bool)) -> DeviceResult<()> {
        let value = led.0 as i32 + ((led.1 as i32) << 1) + ((led.2 as i32) << 2);
        self.write_ctrl_file(npu_mgmt::file::DEVICE_LED, &value.to_string())
    }

    fn ctrl_ne_dtm_policy(&self, policy: npu_mgmt::DtmPolicy) -> DeviceResult<()> {
        self.write_ctrl_file(npu_mgmt::file::NE_DTM_POLICY, &(policy as u8).to_string())
    }

    fn ctrl_performance_level(&self, level: npu_mgmt::PerfLevel) -> DeviceResult<()> {
        self.write_ctrl_file(
            npu_mgmt::file::PERFORMANCE_LEVEL,
            &(level as u8).to_string(),
        )
    }

    fn ctrl_performance_mode(&self, mode: npu_mgmt::PerfMode) -> DeviceResult<()> {
        self.write_ctrl_file(npu_mgmt::file::PERFORMANCE_MODE, &(mode as u8).to_string())
    }
}

impl DevicePerf for WarboyInner {
    fn get_performance_counter(&self, file: &DeviceFile) -> DeviceResult<PerformanceCounter> {
        let dev_name = file.filename();
        let path = self
            .sysfs
            .join(format!("class/npu_mgmt/{dev_name}/perf_regs"));
        PerformanceCounter::read(path).map_err(DeviceError::performance_counter_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warboy_inner() -> eyre::Result<()> {
        let device = WarboyInner::new(0, PathBuf::from("/sys"));
        Ok(())
    }
}
