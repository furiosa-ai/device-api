mod builder;
mod find;
mod parse;

pub use builder::DeviceConfigBuilder;
pub(crate) use find::{expand_status, find_devices_in};
pub use parse::DeviceConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::list_devices_with;

    #[tokio::test]
    async fn test_find_devices() -> DeviceResult<()> {
        // test directory contains 2 warboy NPUs
        let devices = list_devices_with("test_data/test-0/dev", "test_data/test-0/sys").await?;
        let devices_with_statuses = expand_status(devices).await?;

        // try lookup 4 different single cores
        let config = DeviceConfig::warboy().single().count(4);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found.len(), 4);
        assert_eq!(found[0].filename(), "npu0pe0");
        assert_eq!(found[1].filename(), "npu0pe1");
        assert_eq!(found[2].filename(), "npu1pe0");
        assert_eq!(found[3].filename(), "npu1pe1");

        // looking for 5 different cores should fail
        let config = DeviceConfig::warboy().single().count(5);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found, vec![]);

        // try lookup 2 different fused cores
        let config = DeviceConfig::warboy().fused().count(2);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].filename(), "npu0pe0-1");
        assert_eq!(found[1].filename(), "npu1pe0-1");

        // looking for 3 different fused cores should fail
        let config = DeviceConfig::warboy().fused().count(3);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found, vec![]);

        Ok(())
    }

    #[test]
    fn test_config_from_named_text_repr() -> Result<(), nom::Err<()>> {
        assert!("0:".parse::<DeviceConfig>().is_err());
        assert!(":0".parse::<DeviceConfig>().is_err());
        assert!("0:0-1-".parse::<DeviceConfig>().is_err());
        assert!("0:1-0".parse::<DeviceConfig>().is_err());

        assert_eq!(
            "0".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::All
            })
        );
        assert_eq!(
            "1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 1,
                core_range: CoreRange::All
            })
        );
        assert_eq!(
            "0:0".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::Range((0, 0))
            })
        );
        assert_eq!(
            "0:1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::Range((1, 1))
            })
        );
        assert_eq!(
            "1:1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 1,
                core_range: CoreRange::Range((1, 1))
            })
        );
        assert_eq!(
            "0:0-1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::Range((0, 1))
            })
        );

        Ok(())
    }

    #[test]
    fn test_config_from_unnamed_text_repr() -> Result<(), nom::Err<()>> {
        assert!("warboy".parse::<DeviceConfig>().is_err());
        assert!("warboy*".parse::<DeviceConfig>().is_err());
        assert!("*1".parse::<DeviceConfig>().is_err());
        assert!("some_npu*10".parse::<DeviceConfig>().is_err());
        assert!("warboy(2*10".parse::<DeviceConfig>().is_err());
        assert_eq!(
            "warboy(1)*2".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Unnamed {
                arch: Arch::Warboy,
                core_num: 1,
                mode: DeviceMode::Single,
                count: 2
            })
        );
        assert_eq!(
            "warboy(2)*4".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Unnamed {
                arch: Arch::Warboy,
                core_num: 2,
                mode: DeviceMode::Fusion,
                count: 4
            })
        );
        assert_eq!(
            "warboy*12".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Unnamed {
                arch: Arch::Warboy,
                core_num: 1,
                mode: DeviceMode::Single,
                count: 12
            })
        );
        // assert!("npu*10".parse::<DeviceConfig>().is_ok());

        Ok(())
    }

    #[test]
    fn test_config_symmetric_display() -> Result<(), nom::Err<()>> {
        assert_eq!("0".parse::<DeviceConfig>()?.to_string(), "0");
        assert_eq!("1".parse::<DeviceConfig>()?.to_string(), "1");
        assert_eq!("0:0".parse::<DeviceConfig>()?.to_string(), "0:0");
        assert_eq!("0:1".parse::<DeviceConfig>()?.to_string(), "0:1");
        assert_eq!("1:0".parse::<DeviceConfig>()?.to_string(), "1:0");
        assert_eq!("0:0-1".parse::<DeviceConfig>()?.to_string(), "0:0-1");

        assert_eq!(
            "warboy(1)*2".parse::<DeviceConfig>()?.to_string(),
            "warboy(1)*2"
        );
        assert_eq!(
            "warboy(2)*4".parse::<DeviceConfig>()?.to_string(),
            "warboy(2)*4"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_named_config_fit() -> DeviceResult<()> {
        let config = "0:0".parse::<DeviceConfig>().unwrap();
        let npu0pe0 = crate::get_device_with("test_data/test-0/dev", "npu0pe0").await?;
        let npu0pe1 = crate::get_device_with("test_data/test-0/dev", "npu0pe1").await?;
        let npu0pe0_1 = crate::get_device_with("test_data/test-0/dev", "npu0pe0-1").await?;
        let npu1pe0 = crate::get_device_with("test_data/test-0/dev", "npu0pe1").await?;

        assert_eq!(config.count(), 1);

        assert!(config.fit(Arch::Warboy, &npu0pe0));
        assert!(!config.fit(Arch::Warboy, &npu0pe1));
        assert!(!config.fit(Arch::Warboy, &npu0pe0_1));
        assert!(!config.fit(Arch::Warboy, &npu1pe0));

        Ok(())
    }

    #[tokio::test]
    async fn test_unnamed_config_fit() -> DeviceResult<()> {
        let config = "warboy(1)*2".parse::<DeviceConfig>().unwrap();

        assert_eq!(config.count(), 2);

        let npu0pe0 = crate::get_device_with("test_data/test-0/dev", "npu0pe0").await?;
        let npu0pe1 = crate::get_device_with("test_data/test-0/dev", "npu0pe1").await?;
        let npu0pe0_1 = crate::get_device_with("test_data/test-0/dev", "npu0pe0-1").await?;

        assert!(config.fit(Arch::Warboy, &npu0pe0));
        assert!(config.fit(Arch::Warboy, &npu0pe1));
        assert!(!config.fit(Arch::Renegade, &npu0pe0));
        assert!(!config.fit(Arch::Warboy, &npu0pe0_1));

        Ok(())
    }
}
