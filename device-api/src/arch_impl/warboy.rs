use std::collections::HashMap;
use std::path::{Path, PathBuf};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::device::{DeviceCtrl, DeviceInner, DeviceMgmt, DevicePerf};
use crate::error::DeviceResult;
use crate::perf_regs::PerformanceCounter;
use crate::sysfs::npu_mgmt;
use crate::{Arch, ClockFrequency, DeviceError, DeviceFile};

#[derive(Clone)]
pub struct WarboyInner {
    arch: Arch,
    device_index: u8,
    sysfs: PathBuf,
    mgmt_root: PathBuf,
    mgmt_cache: HashMap<StaticMgmtFile, String>,
}

impl WarboyInner {
    pub fn new(arch: Arch, device_index: u8, sysfs: PathBuf) -> DeviceResult<Self> {
        let mgmt_root = sysfs.join(format!("class/npu_mgmt/npu{device_index}_mgmt"));
        let m: DeviceResult<HashMap<_, _>> = StaticMgmtFile::iter()
            .map(|key| {
                let value = npu_mgmt::read_mgmt_to_string(&mgmt_root, key.filename())?;
                Ok((key, value))
            })
            .collect();
        let mgmt_cache = m?;

        Ok(WarboyInner {
            arch,
            device_index,
            sysfs,
            mgmt_root,
            mgmt_cache,
        })
    }

    fn read_mgmt_to_string<P: AsRef<Path>>(&self, file: P) -> DeviceResult<String> {
        npu_mgmt::read_mgmt_to_string(&self.mgmt_root, file).map_err(|e| e.into())
    }

    fn write_ctrl_file<P: AsRef<Path>>(&self, file: P, contents: &str) -> DeviceResult<()> {
        let path = self.mgmt_root.join(file);
        std::fs::write(path, contents)?;
        Ok(())
    }

    fn get_mgmt_cache(&self, file: StaticMgmtFile) -> String {
        self.mgmt_cache
            .get(&file)
            .unwrap_or(&Default::default())
            .clone()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, EnumIter)]
enum StaticMgmtFile {
    BusName,
    Dev,
    DeviceSN,
    DeviceUUID,
    FWVersion,
    Version,
}

impl StaticMgmtFile {
    fn filename(&self) -> &'static str {
        match self {
            StaticMgmtFile::BusName => npu_mgmt::file::BUS_NAME,
            StaticMgmtFile::Dev => npu_mgmt::file::DEV,
            StaticMgmtFile::DeviceSN => npu_mgmt::file::DEVICE_SN,
            StaticMgmtFile::DeviceUUID => npu_mgmt::file::DEVICE_UUID,
            StaticMgmtFile::FWVersion => npu_mgmt::file::FW_VERSION,
            StaticMgmtFile::Version => npu_mgmt::file::VERSION,
        }
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

    fn arch(&self) -> Arch {
        self.arch
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

    fn busname(&self) -> String {
        self.get_mgmt_cache(StaticMgmtFile::BusName)
    }

    fn pci_dev(&self) -> String {
        self.get_mgmt_cache(StaticMgmtFile::Dev)
    }

    fn device_sn(&self) -> String {
        self.get_mgmt_cache(StaticMgmtFile::DeviceSN)
    }

    fn device_uuid(&self) -> String {
        self.get_mgmt_cache(StaticMgmtFile::DeviceUUID)
    }

    fn firmware_version(&self) -> String {
        self.get_mgmt_cache(StaticMgmtFile::FWVersion)
    }

    fn driver_version(&self) -> String {
        self.get_mgmt_cache(StaticMgmtFile::Version)
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
    fn test_warboy_inner_functionality() -> eyre::Result<()> {
        let device = WarboyInner::new(Arch::WarboyB0, 0, PathBuf::from("../test_data/test-1/sys"))?;

        assert_eq!(device.device_index(), 0);
        assert_eq!(device.arch(), Arch::WarboyB0);
        assert!(device.alive()?);
        assert_eq!(device.atr_error()?.len(), 9);
        assert_eq!(device.busname(), "0000:6d:00.0");
        assert_eq!(device.pci_dev(), "000:0");
        assert_eq!(device.device_sn(), "WBYB0000000000000");
        assert_eq!(device.device_uuid(), "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA");
        assert_eq!(device.firmware_version(), "1.6.0, c1bebfd");
        assert_eq!(device.driver_version(), "1.0.0, 0000000");
        assert_eq!(device.heartbeat()?, 42);

        Ok(())
    }
}
