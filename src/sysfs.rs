pub mod npu_mgmt {
    use std::collections::HashMap;
    use std::io;
    use std::path::{Path, PathBuf};

    #[derive(Copy, Clone, Debug)]
    pub enum PerfMode {
        Full2 = 5,
        Full1 = 4,
        Normal2 = 3,
        Normal1 = 2,
        Half = 1,
        Low = 0,
    }

    #[derive(Copy, Clone, Debug)]
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

    pub(crate) static ALIVE: &str = "alive";
    pub(crate) static ATR_ERROR: &str = "atr_error";
    pub(crate) static BUSNAME: &str = "busname";
    pub(crate) static CUR_PE_IDS: &str = "cur_pe_ids";
    pub(crate) static DEV: &str = "dev";
    pub(crate) static DEVICE_LED: &str = "device_led";
    pub(crate) static DEVICE_STATE: &str = "device_state";
    pub(crate) static DEVICE_TYPE: &str = "device_type";
    pub(crate) static DEVICE_UUID: &str = "device_uuid";
    pub(crate) static EVB_REV: &str = "evb_rev";
    pub(crate) static FW_VERSION: &str = "fw_version";
    pub(crate) static HEARTBEAT: &str = "heartbeat";
    pub(crate) static NE_CLK_FREQ_INFO: &str = "ne_clk_freq_info";
    pub(crate) static NE_CLOCK: &str = "ne_clock";
    pub(crate) static NE_DTM_POLICY: &str = "ne_dtm_policy";
    pub(crate) static PERFORMANCE_LEVEL: &str = "performance_level";
    pub(crate) static PERFORMANCE_MODE: &str = "performance_mode";
    pub(crate) static PLATFORM_TYPE: &str = "platform_type";
    pub(crate) static SOC_REV: &str = "soc_rev";
    pub(crate) static SOC_UID: &str = "soc_uid";
    pub(crate) static VERSION: &str = "version";

    pub(crate) static MGMT_FILES: &[(&str, bool)] = &[
        (ALIVE, false),
        (ATR_ERROR, false),
        (BUSNAME, true),
        (CUR_PE_IDS, false),
        (DEV, true),
        (DEVICE_STATE, false),
        (DEVICE_TYPE, true),
        (DEVICE_UUID, false),
        (EVB_REV, false),
        (FW_VERSION, false),
        (HEARTBEAT, false),
        (NE_CLK_FREQ_INFO, false),
        (NE_DTM_POLICY, false),
        (PERFORMANCE_LEVEL, false),
        (PERFORMANCE_MODE, false),
        (PLATFORM_TYPE, false),
        (SOC_REV, false),
        (SOC_UID, false),
        (VERSION, false),
    ];

    pub(crate) static CTRL_FILES: &[&str] = &[
        DEVICE_LED,
        NE_CLOCK,
        NE_DTM_POLICY,
        PERFORMANCE_LEVEL,
        PERFORMANCE_MODE,
    ];

    pub(crate) fn path<P: AsRef<Path>>(base_dir: P, file: &str, idx: u8) -> PathBuf {
        base_dir
            .as_ref()
            .join(format!("class/npu_mgmt/npu{}_mgmt/{}", idx, file))
    }

    /// It can be used to check `platform_type`.
    pub(crate) fn is_furiosa_platform(contents: &str) -> bool {
        let contents = contents.trim();
        contents == "FuriosaAI" || contents == "VITIS"
    }

    pub(crate) fn read_mgmt_file<P: AsRef<Path>>(
        sysfs: P,
        mgmt_file: &str,
        idx: u8,
    ) -> io::Result<String> {
        let path = path(sysfs, mgmt_file, idx);
        std::fs::read_to_string(&path).map(|s| s.trim().to_string())
    }

    pub(crate) fn read_mgmt_files<P: AsRef<Path>>(
        sysfs: P,
        idx: u8,
    ) -> io::Result<HashMap<&'static str, String>> {
        let mut mgmt_files: HashMap<&'static str, String> = HashMap::new();
        for (mgmt_file, required) in MGMT_FILES {
            if !required {
                continue;
            }

            let contents = read_mgmt_file(&sysfs, mgmt_file, idx)?;
            if mgmt_files.insert(mgmt_file, contents).is_some() {
                unreachable!("duplicate key: {}", mgmt_file);
            }
        }
        Ok(mgmt_files)
    }

    pub(crate) fn write_ctrl_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        sysfs: P,
        ctrl_file: &str,
        idx: u8,
        contents: C,
    ) -> io::Result<()> {
        let path = path(sysfs, ctrl_file, idx);
        std::fs::write(&path, contents)
    }
}

pub(crate) mod hwmon {
    use std::path::PathBuf;

    pub fn path(base_dir: &str, bdf: &str) -> PathBuf {
        PathBuf::from(format!("{}/bus/pci/devices/{}/hwmon", base_dir, bdf.trim()))
    }
}
