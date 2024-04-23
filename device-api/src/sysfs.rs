pub mod npu_mgmt {
    use std::collections::HashMap;
    use std::fs;
    use std::hash::Hash;
    use std::io;
    use std::path::Path;
    use std::path::PathBuf;

    use crate::DeviceResult;

    #[allow(dead_code)]
    pub mod file {
        pub const ALIVE: &str = "alive";
        pub const ATR_ERROR: &str = "atr_error";
        pub const BUS_NAME: &str = "busname";
        pub const DEV: &str = "dev";
        pub const DEVICE_LED: &str = "device_led";
        pub const DEVICE_LEDS: &str = "device_leds";
        pub const DEVICE_SN: &str = "device_sn";
        pub const DEVICE_STATE: &str = "device_state";
        pub const DEVICE_TYPE: &str = "device_type";
        pub const DEVICE_UUID: &str = "device_uuid";
        pub const FW_VERSION: &str = "fw_version";
        pub const HEARTBEAT: &str = "heartbeat";
        pub const NE_CLK_FREQ_INFO: &str = "ne_clk_freq_info";
        pub const NE_DTM_POLICY: &str = "ne_dtm_policy";
        pub const NPU_CLOCKS: &str = "npu_clocks";
        pub const PERFORMANCE_LEVEL: &str = "performance_level";
        pub const PERFORMANCE_MODE: &str = "performance_mode";
        pub const PLATFORM_TYPE: &str = "platform_type";
        pub const SOC_ID: &str = "soc_id";
        pub const SOC_REV: &str = "soc_rev";
        pub const SOC_UID: &str = "soc_uid";
        pub const VERSION: &str = "version";
    }

    #[derive(Copy, Clone, Debug)]
    #[allow(dead_code)]
    pub enum Toggle {
        Enable = 1,
        Disable = 0,
    }

    #[derive(Copy, Clone, Debug)]
    #[allow(dead_code)]
    pub enum DtmPolicy {
        OnDemand = 1,
        Conservative = 0,
    }

    #[derive(Copy, Clone, Debug)]
    #[allow(dead_code)]
    pub enum PerfMode {
        Normal2 = 3,
        Normal1 = 2,
        Half = 1,
        Low = 0,
    }

    #[derive(Copy, Clone, Debug)]
    #[allow(dead_code)]
    pub enum PerfLevel {
        Level0 = 0,
        Level1 = 1,
        Level2 = 2,
        Level3 = 3,
        Level4 = 4,
        Level5 = 5,
        Level6 = 6,
        Level7 = 7,
        Level8 = 8,
        Level9 = 9,
        Level10 = 10,
        Level11 = 11,
        Level12 = 12,
        Level13 = 13,
        Level14 = 14,
        Level15 = 15,
    }

    pub(crate) fn read_mgmt_to_string<P: AsRef<Path>, F: AsRef<Path>>(
        mgmt_root: P,
        file: F,
    ) -> io::Result<String> {
        let path = mgmt_root.as_ref().join(file);
        fs::read_to_string(path).map(|s| s.trim_end().to_string())
    }

    /// It can be used to check `platform_type`.
    pub(crate) fn is_furiosa_platform(contents: &str) -> bool {
        let contents = contents.trim();
        contents == "FuriosaAI"
    }

    pub(crate) fn build_atr_error_map<S: AsRef<str>>(contents: S) -> HashMap<String, u32> {
        let mut error_map = HashMap::new();

        let contents = contents.as_ref().trim();
        for line in contents.lines() {
            let line = line.trim();

            if let Some((key, value)) = line.split_once(':') {
                if let Ok(value) = value.trim().parse::<u32>() {
                    let key = key.trim().to_lowercase().replace(' ', "_");

                    error_map.insert(key, value);
                }
            }
        }

        error_map
    }

    pub(crate) fn parse_zero_or_one_to_bool<S: AsRef<str>>(contents: S) -> Option<bool> {
        let contents = contents.as_ref().trim();
        match contents {
            "0" => Some(false),
            "1" => Some(true),
            _ => None,
        }
    }

    pub(crate) trait MgmtFileIO {
        fn mgmt_root(&self) -> PathBuf;

        fn read_mgmt_to_string<P: AsRef<Path>>(&self, file: P) -> DeviceResult<String> {
            read_mgmt_to_string(self.mgmt_root(), file).map_err(|e| e.into())
        }

        fn write_ctrl_file<P: AsRef<Path>>(&self, file: P, contents: &str) -> DeviceResult<()> {
            let path = &self.mgmt_root().join(file);
            std::fs::write(path, contents)?;
            Ok(())
        }
    }

    pub(crate) trait MgmtFile {
        fn filename(&self) -> &'static str;
    }

    #[derive(Clone, Debug)]
    pub(crate) struct MgmtCache<K: Eq + Hash + MgmtFile> {
        cache: HashMap<K, String>,
    }

    impl<K: Eq + Hash + MgmtFile> MgmtCache<K> {
        pub fn init<P: AsRef<Path>>(
            mgmt_root: P,
            keys: impl Iterator<Item = K>,
        ) -> io::Result<Self> {
            let cache: io::Result<HashMap<_, _>> = keys
                .map(|key| {
                    let value = read_mgmt_to_string(&mgmt_root, key.filename())?;
                    Ok((key, value))
                })
                .collect();

            let cache = cache?;
            Ok(MgmtCache { cache })
        }

        pub fn get(&self, key: &K) -> String {
            self.cache.get(key).unwrap_or(&Default::default()).clone()
        }
    }
}

// XXX(n0gu): warboy and renegade share the same implementation, but this may change in the future devices.
pub(crate) mod pci {
    pub(crate) mod numa {
        use std::io;
        use std::path::{Path, PathBuf};

        pub(crate) fn path<P: AsRef<Path>>(base_dir: P, bdf: &str) -> PathBuf {
            base_dir
                .as_ref()
                .join(format!("bus/pci/devices/{}/numa_node", bdf.trim()))
        }

        pub(crate) fn read_numa_node<P: AsRef<Path>>(sysfs: P, bdf: &str) -> io::Result<String> {
            let path = path(sysfs, bdf);
            std::fs::read_to_string(path).map(|s| s.trim().to_string())
        }
    }

    pub(crate) mod hwmon {
        use std::path::PathBuf;

        pub fn path(base_dir: &str, bdf: &str) -> PathBuf {
            PathBuf::from(format!("{}/bus/pci/devices/{}/hwmon", base_dir, bdf.trim()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_atr_error_map() {
        let case1 = r"AXI Post Error: 0
AXI Fetch Error: 0
AXI Discard Error: 0
AXI Doorbell done: 0
PCIe Post Error: 0
PCIe Fetch Error: 0
PCIe Discard Error: 0
PCIe Doorbell done: 0
Device Error: 0";

        let res = npu_mgmt::build_atr_error_map(case1);
        assert_eq!(res.len(), 9);
        assert_eq!(res.get("device_error"), Some(0_u32).as_ref());
        assert_eq!(res.get("device_error"), Some(0_u32).as_ref());
        assert_eq!(res.get("axi_fetch_error"), Some(0_u32).as_ref());
        assert_eq!(res.get("pcie_fetch_error"), Some(0_u32).as_ref());
    }

    #[test]
    fn test_parse_zero_or_one_to_bool() {
        let case1 = "1";
        let res1 = npu_mgmt::parse_zero_or_one_to_bool(case1);
        assert!(res1.is_some());
        assert!(res1.unwrap());

        let case2 = "0";
        let res2 = npu_mgmt::parse_zero_or_one_to_bool(case2);
        assert!(res2.is_some());
        assert!(!res2.unwrap());

        let case3 = "";
        let res3 = npu_mgmt::parse_zero_or_one_to_bool(case3);
        assert!(res3.is_none());
    }
}
