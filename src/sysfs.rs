pub(crate) mod npu_mgmt {
    use std::path::{Path, PathBuf};

    // commented lines are write-only files
    pub(crate) static ALIVE: &str = "alive";
    pub(crate) static ATR_ERROR: &str = "atr_error";
    pub(crate) static BUSNAME: &str = "busname";
    pub(crate) static CUR_PE_IDS: &str = "cur_pe_ids";
    pub(crate) static DEV: &str = "dev";
    //    pub(crate) static DEVICE_LED: &str = "device_led";
    pub(crate) static DEVICE_STATE: &str = "device_state";
    pub(crate) static DEVICE_TYPE: &str = "device_type";
    pub(crate) static DEVICE_UUID: &str = "device_uuid";
    pub(crate) static EVB_REV: &str = "evb_rev";
    pub(crate) static FW_VERSION: &str = "fw_version";
    pub(crate) static HEARTBEAT: &str = "heartbeat";
    pub(crate) static NE_CLK_FREQ_INFO: &str = "ne_clk_freq_info";
    //    pub(crate) static NE_CLOCK: &str = "ne_clock";
    pub(crate) static NE_DTM_POLICY: &str = "ne_dtm_policy";
    //    pub(crate) static NEW_PE_IDS: &str = "new_pe_ids";
    pub(crate) static PERFORMANCE_LEVEL: &str = "performance_level";
    pub(crate) static PERFORMANCE_MODE: &str = "performance_mode";
    pub(crate) static PLATFORM_TYPE: &str = "platform_type";
    pub(crate) static REBOOT_REASON: &str = "reboot_reason";
    //    pub(crate) static REMOVE_PE_IDS: &str = "remove_pe_ids";
    //    pub(crate) static RESET: &str = "reset";
    pub(crate) static SOC_REV: &str = "soc_rev";
    pub(crate) static SOC_UID: &str = "soc_uid";
    //    pub(crate) static TTY_LOCK: &str = "tty_lock";
    pub(crate) static UEVENT: &str = "uevent";
    pub(crate) static VERSION: &str = "version";

    pub fn path<P: AsRef<Path>>(base_dir: P, file: &str, idx: u8) -> PathBuf {
        base_dir
            .as_ref()
            .join(format!("class/npu_mgmt/npu{}_mgmt/{}", idx, file))
    }

    /// It can be used to check `platform_type`.
    pub fn is_furiosa_platform(contents: &str) -> bool {
        let contents = contents.trim();
        contents == "FuriosaAI" || contents == "VITIS"
    }
}

pub(crate) mod hwmon {
    use std::path::PathBuf;

    pub fn path(base_dir: &str, bdf: &str) -> PathBuf {
        PathBuf::from(format!("{}/bus/pci/devices/{}/hwmon", base_dir, bdf.trim()))
    }
}
