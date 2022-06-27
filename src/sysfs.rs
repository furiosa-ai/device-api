pub(crate) mod npu_mgmt {
    use std::path::PathBuf;

    pub(crate) static PLATFORM_TYPE: &str = "platform_type";
    pub(crate) static DEVICE_TYPE: &str = "device_type";
    pub(crate) static DEV: &str = "dev";
    pub(crate) static BUSNAME: &str = "busname";
    pub(crate) static FW_VERSION: &str = "fw_version";

    pub fn path(base_dir: &str, file: &str, idx: u8) -> PathBuf {
        PathBuf::from(format!(
            "{}/class/npu_mgmt/npu{}_mgmt/{}",
            base_dir, idx, file
        ))
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
