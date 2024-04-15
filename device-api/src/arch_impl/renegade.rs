use std::collections::HashMap;
use std::path::{Path, PathBuf};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::device::{DeviceCtrl, DeviceInner, DeviceMgmt, DevicePerf};
use crate::error::DeviceResult;
use crate::sysfs::npu_mgmt;
use crate::Arch;
use crate::ClockFrequency;
use crate::DeviceError;

#[derive(Clone)]
pub struct RenegadeInner {
    arch: Arch,
    device_index: u8,
    sysfs: PathBuf,
    mgmt_root: PathBuf,
    mgmt_cache: HashMap<StaticMgmtFile, String>,
}

impl RenegadeInner {
    pub fn new(arch: Arch, device_index: u8, sysfs: PathBuf) -> DeviceResult<Self> {
        let mgmt_root = sysfs.join(format!(
            "class/renegade_mgmt/renegade!npu{device_index}mgmt"
        ));
        let m: DeviceResult<HashMap<_, _>> = StaticMgmtFile::iter()
            .map(|key| {
                let value = npu_mgmt::read_mgmt_to_string(&mgmt_root, key.filename())?;
                Ok((key, value))
            })
            .collect();
        let mgmt_cache = m?;

        Ok(RenegadeInner {
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

    #[allow(dead_code)]
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
        self.arch
    }

    fn alive(&self) -> DeviceResult<bool> {
        self.read_mgmt_to_string(npu_mgmt::file::DEVICE_STATE)
            .map(|v| v == "good")
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
        let device =
            RenegadeInner::new(Arch::Renegade, 0, PathBuf::from("../test_data/test-1/sys"))?;

        assert_eq!(device.device_index(), 0);
        assert_eq!(device.arch(), Arch::Renegade);
        assert!(device.alive()?);
        assert_eq!(device.atr_error()?.len(), 9);
        assert_eq!(device.busname(), "0000:00:03.0");
        assert_eq!(device.pci_dev(), "235:0");
        assert_eq!(device.device_sn(), "");
        assert_eq!(device.device_uuid(), "82540B87-1055-48C6-AAB1-C4CC84672C71");
        assert_eq!(device.firmware_version(), "");
        assert_eq!(device.driver_version(), "1.0.0, 0abcdef");
        assert_eq!(device.heartbeat()?, 0);

        Ok(())
    }
}