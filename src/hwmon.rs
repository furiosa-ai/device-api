/* https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface */
/* The common scheme for files naming is: <type><number>_<item>. */

use std::{collections::HashMap, path::PathBuf, str::FromStr};

use itertools::Itertools;
use thiserror::Error;
use tokio::fs::DirEntry;

use crate::{sysfs::npu_mgmt, DeviceError, DeviceResult};

/// An error that occurred during parsing or retrieving hwmon sensors.
#[derive(Debug, Error)]
pub enum HwmonError {
    #[error("Unsupported type: {name}")]
    UnsupportedType { name: String },
    #[error("No metric element file")]
    NoMetricElementFile,
    #[error("Not found item: {name}")]
    NotFoundItemName { name: String },
    #[error("Unexpected value format: {value}")]
    UnexpectedValueFormat { value: String },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum HwmonType {
    Current,
    Voltage,
    Power,
    Temperature,
}

impl FromStr for HwmonType {
    type Err = HwmonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "curr" => Ok(HwmonType::Current),
            "in" => Ok(HwmonType::Voltage),
            "power" => Ok(HwmonType::Power),
            "temp" => Ok(HwmonType::Temperature),
            _ => Err(HwmonError::UnsupportedType {
                name: String::from(s),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct HwmonMetricType {
    hwmon_type: HwmonType,
    idx: u8,
}

#[derive(Debug)]
struct HwmonMetricItem {
    item_name: String,
    path: PathBuf,
}

#[derive(Debug)]
struct HwmonMetricEntry {
    metric_type: HwmonMetricType,
    metric_item: HwmonMetricItem,
}

impl TryFrom<DirEntry> for HwmonMetricEntry {
    type Error = HwmonError;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let filename = value.file_name().to_string_lossy().to_string();
        let underscore_pos = filename.find('_').ok_or(HwmonError::NoMetricElementFile)?;

        let (metric_type_str, metric_item_str) = filename.split_at(underscore_pos);
        let idx_pos = metric_type_str
            .find(|c: char| c.is_digit(10))
            .ok_or(HwmonError::NoMetricElementFile)?;

        let (name_str, idx_str) = metric_type_str.split_at(idx_pos);
        let idx = idx_str
            .parse::<u8>()
            .map_err(|_| HwmonError::NoMetricElementFile)?;
        let metric_type = HwmonType::from_str(name_str)?;

        Ok(HwmonMetricEntry {
            metric_type: HwmonMetricType {
                hwmon_type: metric_type,
                idx,
            },
            metric_item: HwmonMetricItem {
                item_name: metric_item_str[1..].to_string(),
                path: value.path(),
            },
        })
    }
}

#[derive(Debug)]
struct NpuHwmonSensor {
    name: String,
    items: HashMap<String, PathBuf>,
}

impl NpuHwmonSensor {
    fn new(name: String, items: Vec<HwmonMetricItem>) -> Self {
        let mut map = HashMap::new();

        for item in items {
            map.insert(item.item_name, item.path);
        }

        Self { name, items: map }
    }

    pub async fn read_item(&self, item_name: &str) -> DeviceResult<(String, String)> {
        if let Some(path) = self.items.get(item_name) {
            let value = tokio::fs::read_to_string(path).await?;

            Ok((self.name.clone(), value.trim().to_string()))
        } else {
            Err(DeviceError::from(HwmonError::NotFoundItemName {
                name: String::from(item_name),
            }))
        }
    }
}

struct NpuHwmonSensorContainer(HashMap<HwmonType, Vec<NpuHwmonSensor>>);

impl NpuHwmonSensorContainer {
    pub async fn new(sysfs_root: &str, idx: u8) -> DeviceResult<Self> {
        let path = Self::build_path(sysfs_root, idx).await?;
        let entries = Self::fetch_entries(path).await?;
        let value_map = Self::build_value_map(entries).await;

        let sensors: HashMap<HwmonType, Vec<NpuHwmonSensor>> = value_map
            .into_iter()
            .map(|(hwmon_type, v)| {
                let metrics: Vec<NpuHwmonSensor> = v
                    .into_iter()
                    .map(|(sensor_name, items)| NpuHwmonSensor::new(sensor_name, items))
                    .collect();

                (hwmon_type, metrics)
            })
            .collect();

        Ok(NpuHwmonSensorContainer(sensors))
    }

    pub fn get(&self, t: &HwmonType) -> Option<&Vec<NpuHwmonSensor>> {
        self.0.get(t)
    }

    async fn build_path(sysfs_root: &str, idx: u8) -> DeviceResult<String> {
        let path = npu_mgmt::path(sysfs_root, "busname", idx);
        let bdf = tokio::fs::read_to_string(&path).await?;
        Ok(format!(
            "{}/bus/pci/devices/{}/hwmon",
            sysfs_root,
            bdf.trim()
        ))
    }

    async fn fetch_entries(path: String) -> DeviceResult<Vec<HwmonMetricEntry>> {
        let mut vec = vec![];

        let mut read_dir = tokio::fs::read_dir(&path).await?;

        // Caution: Assume that there is only one 'hwmon' per device
        if let Some(entry) = read_dir.next_entry().await? {
            let hwmon_name = entry.file_name();
            let path = format!("{}/{}", path, hwmon_name.to_string_lossy());

            let mut read_dir = tokio::fs::read_dir(&path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                // ignore error: not subject to collection
                if let Ok(metric) = HwmonMetricEntry::try_from(entry) {
                    vec.push(metric);
                }
            }
        }

        Ok(vec)
    }

    async fn build_value_map(
        entries: Vec<HwmonMetricEntry>,
    ) -> HashMap<HwmonType, Vec<(String, Vec<HwmonMetricItem>)>> {
        let (labels, metrics): (Vec<_>, Vec<_>) = entries
            .into_iter()
            .partition(|entry| entry.metric_item.item_name == "label");
        let label_map = Self::build_label_map(labels).await;

        let mut map_by_metric_type = HashMap::new();
        for entry in metrics {
            map_by_metric_type
                .entry(entry.metric_type)
                .or_insert_with(Vec::new)
                .push(entry.metric_item);
        }

        let labelled_metrics: Vec<(HwmonType, String, Vec<HwmonMetricItem>)> = map_by_metric_type
            .into_iter()
            .sorted_by(|a, b| a.0.idx.cmp(&b.0.idx))
            .map(|(k, items)| {
                let label = label_map
                    .get(&k)
                    .cloned()
                    .unwrap_or_else(|| k.idx.to_string());

                (k.hwmon_type, label, items)
            })
            .collect();

        let mut res = HashMap::new();
        for (t, label, items) in labelled_metrics {
            res.entry(t).or_insert_with(Vec::new).push((label, items));
        }

        res
    }

    async fn build_label_map(
        label_entries: Vec<HwmonMetricEntry>,
    ) -> HashMap<HwmonMetricType, String> {
        let mut map = HashMap::new();

        for entry in label_entries {
            if let Ok(text) = tokio::fs::read_to_string(&entry.metric_item.path).await {
                map.insert(entry.metric_type, text.trim().to_string());
            }
        }

        map
    }
}

pub struct NpuHwmonAccessor {
    npu_idx: u8,
    sensors: NpuHwmonSensorContainer,
}

impl NpuHwmonAccessor {
    pub async fn new(sysfs_root: &str, idx: u8) -> DeviceResult<Self> {
        let sensors = NpuHwmonSensorContainer::new(sysfs_root, idx).await?;

        Ok(Self {
            npu_idx: idx,
            sensors,
        })
    }

    pub fn get_npu_idx(&self) -> u8 {
        self.npu_idx
    }

    pub async fn read_currents(&self) -> DeviceResult<Vec<(String, i32)>> {
        self.read_values(HwmonType::Current, "input").await
    }

    pub async fn read_voltages(&self) -> DeviceResult<Vec<(String, i32)>> {
        self.read_values(HwmonType::Voltage, "input").await
    }

    pub async fn read_powers_average(&self) -> DeviceResult<Vec<(String, i32)>> {
        self.read_values(HwmonType::Power, "average").await
    }

    pub async fn read_temperatures(&self) -> DeviceResult<Vec<(String, i32)>> {
        self.read_values(HwmonType::Temperature, "input").await
    }

    async fn read_values(&self, t: HwmonType, name: &str) -> DeviceResult<Vec<(String, i32)>> {
        let mut res = vec![];

        if let Some(metrics) = self.sensors.get(&t) {
            for metric in metrics {
                let (label, value) = metric.read_item(name).await?;
                let value: i32 = value
                    .parse()
                    .map_err(|_| HwmonError::UnexpectedValueFormat { value })?;

                res.push((label, value));
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hwmon_metric_entry_try_from_test() -> DeviceResult<()> {
        let mut dir =
            tokio::fs::read_dir("test_data/test-0/sys/bus/pci/devices/0000:ff:00.0/hwmon/hwmon0")
                .await?;

        if let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            let res = HwmonMetricEntry::try_from(entry)?;
            assert_eq!(
                res.metric_type,
                HwmonMetricType {
                    hwmon_type: HwmonType::Current,
                    idx: 1
                }
            );
            assert_eq!(res.metric_item.item_name, String::from("input"));
            assert_eq!(res.metric_item.path, path);
        }

        Ok(())
    }

    #[tokio::test]
    async fn sensor_build_path_test() -> DeviceResult<()> {
        let path = NpuHwmonSensorContainer::build_path("test_data/test-0/sys", 0).await?;
        assert_eq!(
            path,
            "test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon"
        );

        let err_expected = NpuHwmonSensorContainer::build_path("invalid-path", 99).await;
        assert!(err_expected.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn sensor_fetch_entries_test() -> DeviceResult<()> {
        let path = String::from("test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon");
        let res = NpuHwmonSensorContainer::fetch_entries(path).await?;
        assert_eq!(res.len(), 16);

        let path = String::from("invalid_path");
        let res = NpuHwmonSensorContainer::fetch_entries(path).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn sensor_build_value_map_test() -> DeviceResult<()> {
        let input = vec![];
        let output = NpuHwmonSensorContainer::build_value_map(input).await;
        assert_eq!(output.len(), 0);

        let input = vec![
            HwmonMetricEntry {
                metric_type: HwmonMetricType { hwmon_type: HwmonType::Temperature, idx: 1 },
                metric_item: HwmonMetricItem {
                    item_name: String::from("label"),
                    path: PathBuf::from("test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_label")
                }
            },
            HwmonMetricEntry {
                metric_type: HwmonMetricType { hwmon_type: HwmonType::Temperature, idx: 1 },
                metric_item: HwmonMetricItem {
                    item_name: String::from("input"),
                    path: PathBuf::from("test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_input")
                }
            },
        ];
        let output = NpuHwmonSensorContainer::build_value_map(input).await;
        assert_eq!(output.len(), 1);
        let opt = output.get(&HwmonType::Temperature);
        assert!(opt.is_some());
        let v = opt.unwrap();
        assert_eq!(v.len(), 1);
        let (label, items) = &v[0];
        assert_eq!(label, "Temp1");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "input");

        let input = vec![HwmonMetricEntry {
            metric_type: HwmonMetricType {
                hwmon_type: HwmonType::Temperature,
                idx: 1,
            },
            metric_item: HwmonMetricItem {
                item_name: String::from("input"),
                path: PathBuf::from(
                    "test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_input",
                ),
            },
        }];
        let output = NpuHwmonSensorContainer::build_value_map(input).await;
        assert_eq!(output.len(), 1);
        let opt = output.get(&HwmonType::Temperature);
        assert!(opt.is_some());
        let v = opt.unwrap();
        assert_eq!(v.len(), 1);
        let (label, items) = &v[0];
        assert_eq!(label, "1");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "input");

        Ok(())
    }

    #[tokio::test]
    async fn sensor_build_label_map_test() -> DeviceResult<()> {
        let input = vec![];
        let output = NpuHwmonSensorContainer::build_label_map(input).await;
        assert_eq!(output.len(), 0);

        let input = vec![HwmonMetricEntry {
            metric_type: HwmonMetricType {
                hwmon_type: HwmonType::Temperature,
                idx: 1,
            },
            metric_item: HwmonMetricItem {
                item_name: String::from("label"),
                path: PathBuf::from(
                    "test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_label",
                ),
            },
        }];
        let output = NpuHwmonSensorContainer::build_label_map(input).await;

        assert_eq!(output.len(), 1);
        let res = output.get(&HwmonMetricType {
            hwmon_type: HwmonType::Temperature,
            idx: 1,
        });
        assert!(res.is_some());
        assert_eq!(res.unwrap(), "Temp1");
        assert!(output
            .get(&HwmonMetricType {
                hwmon_type: HwmonType::Temperature,
                idx: 2
            })
            .is_none());
        assert!(output
            .get(&HwmonMetricType {
                hwmon_type: HwmonType::Power,
                idx: 1
            })
            .is_none());

        Ok(())
    }

    #[tokio::test]
    async fn accessor_read_test() -> DeviceResult<()> {
        let accessor = NpuHwmonAccessor::new("test_data/test-0/sys", 0).await?;

        assert_eq!(accessor.get_npu_idx(), 0);

        let currents = accessor.read_currents().await?;
        assert_eq!(currents.len(), 2);
        assert_eq!(currents[0].0, "Current1");
        assert_eq!(currents[0].1, 1000);
        assert_eq!(currents[1].0, "Current2");
        assert_eq!(currents[1].1, 2000);

        let voltages = accessor.read_voltages().await?;
        assert_eq!(voltages.len(), 2);
        assert_eq!(voltages[0].0, "Voltage0");
        assert_eq!(voltages[0].1, 1100);
        assert_eq!(voltages[1].0, "Voltage1");
        assert_eq!(voltages[1].1, 1200);

        let powers = accessor.read_powers_average().await?;
        assert_eq!(powers.len(), 2);
        assert_eq!(powers[0].0, "Power1");
        assert_eq!(powers[0].1, 1111);
        assert_eq!(powers[1].0, "Power2");
        assert_eq!(powers[1].1, 22222);

        let temperature = accessor.read_temperatures().await?;
        assert_eq!(temperature.len(), 2);
        assert_eq!(temperature[0].0, "Temp1");
        assert_eq!(temperature[0].1, 36000);
        assert_eq!(temperature[1].0, "Temp2");
        assert_eq!(temperature[1].1, 37000);

        Ok(())
    }
}
