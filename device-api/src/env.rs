#[cfg(not(test))]
pub fn get_dev_fs(default: &str) -> String {
    default.to_string()
}

#[cfg(not(test))]
pub fn get_sys_fs(default: &str) -> String {
    default.to_string()
}

/// Parse and return dev fs path defined in the FURIOSA_DEV_FS env var for test.
/// Return the default dev fs path, if the env var is not set.
#[cfg(test)]
pub fn get_dev_fs(default: &str) -> String {
    match std::env::var("FURIOSA_DEV_FS") {
        Ok(str) => str,
        _ => default.to_string(),
    }
}

/// Parse and return sys fs path defined in the FURIOSA_sys_FS env var for test.
/// Return the default sys fs path, if the env var is not set.
#[cfg(test)]
pub fn get_sys_fs(default: &str) -> String {
    match std::env::var("FURIOSA_SYS_FS") {
        Ok(str) => str,
        _ => default.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_without_env_var() {
        std::env::remove_var("FURIOSA_DEV_FS");
        std::env::remove_var("FURIOSA_SYS_FS");
        assert_eq!(get_dev_fs("/dev").as_str(), "/dev");
        assert_eq!(get_sys_fs("/sys").as_str(), "/sys");
    }
    #[test]
    fn test_with_env_var() {
        std::env::set_var("FURIOSA_DEV_FS", "/test/dev");
        std::env::set_var("FURIOSA_SYS_FS", "/test/sys");
        assert_eq!(get_dev_fs("/dev").as_str(), "/test/dev");
        assert_eq!(get_sys_fs("/sys").as_str(), "/test/sys");
        std::env::remove_var("FURIOSA_DEV_FS");
        std::env::remove_var("FURIOSA_SYS_FS");
    }
}
