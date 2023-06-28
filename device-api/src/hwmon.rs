/* https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface */
/* The common scheme for files naming is: <type><number>_<item>. */

use std::{collections::HashMap, path::PathBuf, str::FromStr};

use itertools::Itertools;

use crate::sysfs::pci::hwmon;
use crate::{DeviceError, DeviceResult};

pub mod error {
    use std::io;

    use thiserror::Error;

    pub type HwmonResult<T> = Result<T, HwmonError>;

    /// An error that occurred during parsing or retrieving hwmon sensors.
    #[derive(Debug, Error)]
    pub enum HwmonError {
        #[error("IoError: {cause}")]
        IoError { cause: io::Error },
        #[error("Unsupported type: {name}")]
        UnsupportedType { name: String },
        #[error("Invalid file name: {name}")]
        InvalidFileName { name: String },
        #[error("Item Not found: {sensor_name} {item_name}")]
        ItemNameNotFound {
            sensor_name: String,
            item_name: String,
        },
        #[error("Unexpected value format: {sensor_name} {value}")]
        UnexpectedValueFormat { sensor_name: String, value: String },
    }

    impl From<io::Error> for HwmonError {
        fn from(e: io::Error) -> Self {
            Self::IoError { cause: e }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum HwmonType {
    Current,
    Voltage,
    Power,
    Temperature,
}

impl FromStr for HwmonType {
    type Err = error::HwmonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "curr" => Ok(HwmonType::Current),
            "in" => Ok(HwmonType::Voltage),
            "power" => Ok(HwmonType::Power),
            "temp" => Ok(HwmonType::Temperature),
            _ => Err(error::HwmonError::UnsupportedType {
                name: String::from(s),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct MetricType {
    hwmon_type: HwmonType,
    idx: u8,
}

impl TryFrom<&str> for MetricType {
    type Error = error::HwmonError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let idx_pos =
            value
                .find(|c: char| c.is_ascii_digit())
                .ok_or(error::HwmonError::InvalidFileName {
                    name: value.to_string(),
                })?;

        let (name_str, idx_str) = value.split_at(idx_pos);

        let hwmon_type = HwmonType::from_str(name_str)?;

        let idx = idx_str
            .parse::<u8>()
            .map_err(|_| error::HwmonError::InvalidFileName {
                name: value.to_string(),
            })?;
        Ok(MetricType { hwmon_type, idx })
    }
}

#[derive(Debug)]
struct MetricItem {
    item_name: String,
    path: PathBuf,
}

#[derive(Debug)]
struct MetricEntry {
    metric_type: MetricType,
    metric_item: MetricItem,
}

impl TryFrom<std::fs::DirEntry> for MetricEntry {
    type Error = error::HwmonError;

    fn try_from(value: std::fs::DirEntry) -> Result<Self, Self::Error> {
        let filename = value.file_name().to_string_lossy().to_string();

        let (metric_type_str, metric_item_str) =
            filename
                .split_once('_')
                .ok_or_else(|| error::HwmonError::InvalidFileName {
                    name: filename.clone(),
                })?;

        let metric_type = MetricType::try_from(metric_type_str)?;
        let metric_item = MetricItem {
            item_name: metric_item_str.to_string(),
            path: value.path(),
        };

        Ok(MetricEntry {
            metric_type,
            metric_item,
        })
    }
}

impl TryFrom<tokio::fs::DirEntry> for MetricEntry {
    type Error = error::HwmonError;

    fn try_from(value: tokio::fs::DirEntry) -> Result<Self, Self::Error> {
        let filename = value.file_name().to_string_lossy().to_string();

        let (metric_type_str, metric_item_str) =
            filename
                .split_once('_')
                .ok_or_else(|| error::HwmonError::InvalidFileName {
                    name: filename.clone(),
                })?;

        let metric_type = MetricType::try_from(metric_type_str)?;
        let metric_item = MetricItem {
            item_name: metric_item_str.to_string(),
            path: value.path(),
        };

        Ok(MetricEntry {
            metric_type,
            metric_item,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Sensor {
    name: String,
    items: HashMap<String, PathBuf>,
}

impl Sensor {
    fn new(name: String, items: Vec<MetricItem>) -> Self {
        let mut map = HashMap::new();

        for item in items {
            map.insert(item.item_name, item.path);
        }

        Self { name, items: map }
    }

    fn read_blocking(&self, item_name: &str) -> error::HwmonResult<(String, String)> {
        if let Some(path) = self.items.get(item_name) {
            let value = std::fs::read_to_string(path)?;

            Ok((self.name.clone(), value.trim().to_string()))
        } else {
            Err(error::HwmonError::ItemNameNotFound {
                sensor_name: self.name.clone(),
                item_name: item_name.to_string(),
            })
        }
    }

    async fn read_item(&self, item_name: &str) -> error::HwmonResult<(String, String)> {
        if let Some(path) = self.items.get(item_name) {
            let value = tokio::fs::read_to_string(path).await?;

            Ok((self.name.clone(), value.trim().to_string()))
        } else {
            Err(error::HwmonError::ItemNameNotFound {
                sensor_name: self.name.clone(),
                item_name: item_name.to_string(),
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct SensorContainer(pub(crate) HashMap<HwmonType, Vec<Sensor>>);

impl SensorContainer {
    pub(crate) fn new_blocking(base_dir: &str, busname: &str) -> error::HwmonResult<Self> {
        let path = hwmon::path(base_dir, busname);
        let entries = Self::fetch_entries_blocking(path)?;
        let value_map = Self::build_value_map_blocking(entries);

        let sensors: HashMap<HwmonType, Vec<Sensor>> = value_map
            .into_iter()
            .map(|(hwmon_type, v)| {
                let metrics: Vec<Sensor> = v
                    .into_iter()
                    .map(|(sensor_name, items)| Sensor::new(sensor_name, items))
                    .collect();

                (hwmon_type, metrics)
            })
            .collect();

        Ok(SensorContainer(sensors))
    }

    async fn new(base_dir: &str, busname: &str) -> error::HwmonResult<Self> {
        let path = hwmon::path(base_dir, busname);
        let entries = Self::fetch_entries(path).await?;
        let value_map = Self::build_value_map(entries).await;

        let sensors: HashMap<HwmonType, Vec<Sensor>> = value_map
            .into_iter()
            .map(|(hwmon_type, v)| {
                let metrics: Vec<Sensor> = v
                    .into_iter()
                    .map(|(sensor_name, items)| Sensor::new(sensor_name, items))
                    .collect();

                (hwmon_type, metrics)
            })
            .collect();

        Ok(SensorContainer(sensors))
    }

    fn get(&self, t: &HwmonType) -> Option<&Vec<Sensor>> {
        self.0.get(t)
    }

    fn fetch_entries_blocking(mut path: PathBuf) -> error::HwmonResult<Vec<MetricEntry>> {
        let mut vec = vec![];

        let mut read_dir = std::fs::read_dir(&path)?;
        if let Some(entry) = read_dir.next() {
            let entry = entry?;
            // Note: Assume that there is only one 'hwmon' per device
            path.push(entry.file_name().to_string_lossy().as_ref());

            let read_dir = std::fs::read_dir(&path)?;
            for entry in read_dir {
                let entry = entry?;
                // Note: Unrecognized entries are ignored
                if let Ok(metric_entry) = MetricEntry::try_from(entry) {
                    vec.push(metric_entry);
                }
            }
        }

        Ok(vec)
    }

    async fn fetch_entries(mut path: PathBuf) -> error::HwmonResult<Vec<MetricEntry>> {
        let mut vec = vec![];

        let mut read_dir = tokio::fs::read_dir(&path).await?;
        if let Some(entry) = read_dir.next_entry().await? {
            // Note: Assume that there is only one 'hwmon' per device
            path.push(entry.file_name().to_string_lossy().as_ref());

            let mut read_dir = tokio::fs::read_dir(&path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                // Note: Unrecognized entries are ignored
                if let Ok(metric_entry) = MetricEntry::try_from(entry) {
                    vec.push(metric_entry);
                }
            }
        }

        Ok(vec)
    }

    fn build_value_map_blocking(
        entries: Vec<MetricEntry>,
    ) -> HashMap<HwmonType, Vec<(String, Vec<MetricItem>)>> {
        let (labels, metrics): (Vec<_>, Vec<_>) = entries
            .into_iter()
            .partition(|entry| entry.metric_item.item_name == "label");
        let label_map = Self::build_label_map_blocking(labels);

        let mut map_by_metric_type = HashMap::new();
        for entry in metrics {
            map_by_metric_type
                .entry(entry.metric_type)
                .or_insert_with(Vec::new)
                .push(entry.metric_item);
        }

        let labelled_metrics: Vec<(HwmonType, String, Vec<MetricItem>)> = map_by_metric_type
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

    async fn build_value_map(
        entries: Vec<MetricEntry>,
    ) -> HashMap<HwmonType, Vec<(String, Vec<MetricItem>)>> {
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

        let labelled_metrics: Vec<(HwmonType, String, Vec<MetricItem>)> = map_by_metric_type
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

    fn build_label_map_blocking(label_entries: Vec<MetricEntry>) -> HashMap<MetricType, String> {
        let mut map = HashMap::new();

        for entry in label_entries {
            if let Ok(text) = std::fs::read_to_string(&entry.metric_item.path) {
                map.insert(entry.metric_type, text.trim().to_string());
            }
        }

        map
    }

    async fn build_label_map(label_entries: Vec<MetricEntry>) -> HashMap<MetricType, String> {
        let mut map = HashMap::new();

        for entry in label_entries {
            if let Ok(text) = tokio::fs::read_to_string(&entry.metric_item.path).await {
                map.insert(entry.metric_type, text.trim().to_string());
            }
        }

        map
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SensorValue {
    pub label: String,
    pub value: i32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fetcher {
    pub(crate) device_index: u8,
    pub(crate) sensor_container: SensorContainer,
}

impl Fetcher {
    pub(crate) async fn new(base_dir: &str, device_index: u8, busname: &str) -> DeviceResult<Self> {
        let sensor_container = SensorContainer::new(base_dir, busname)
            .await
            .map_err(|e| DeviceError::hwmon_error(device_index, e))?;

        Ok(Self {
            device_index,
            sensor_container,
        })
    }

    pub fn read_currents_blocking(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values_blocking(HwmonType::Current, "input")
    }

    pub fn read_voltages_blocking(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values_blocking(HwmonType::Voltage, "input")
    }

    pub fn read_powers_average_blocking(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values_blocking(HwmonType::Power, "average")
    }

    pub fn read_temperatures_blocking(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values_blocking(HwmonType::Temperature, "input")
    }

    pub async fn read_currents(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values(HwmonType::Current, "input").await
    }

    pub async fn read_voltages(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values(HwmonType::Voltage, "input").await
    }

    pub async fn read_powers_average(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values(HwmonType::Power, "average").await
    }

    pub async fn read_temperatures(&self) -> DeviceResult<Vec<SensorValue>> {
        self.read_values(HwmonType::Temperature, "input").await
    }

    fn read_values_blocking(&self, t: HwmonType, name: &str) -> DeviceResult<Vec<SensorValue>> {
        let mut res = vec![];

        if let Some(sensors) = self.sensor_container.get(&t) {
            for sensor in sensors {
                let (label, value) = sensor
                    .read_blocking(name)
                    .map_err(|e| DeviceError::hwmon_error(self.device_index, e))?;

                let value: i32 = value.parse().map_err(|_| {
                    DeviceError::hwmon_error(
                        self.device_index,
                        error::HwmonError::UnexpectedValueFormat {
                            sensor_name: label.clone(),
                            value,
                        },
                    )
                })?;

                res.push(SensorValue { label, value });
            }
        }

        Ok(res)
    }

    async fn read_values(&self, t: HwmonType, name: &str) -> DeviceResult<Vec<SensorValue>> {
        let mut res = vec![];

        if let Some(sensors) = self.sensor_container.get(&t) {
            for sensor in sensors {
                let (label, value) = sensor
                    .read_item(name)
                    .await
                    .map_err(|e| DeviceError::hwmon_error(self.device_index, e))?;

                let value: i32 = value.parse().map_err(|_| {
                    DeviceError::hwmon_error(
                        self.device_index,
                        error::HwmonError::UnexpectedValueFormat {
                            sensor_name: label.clone(),
                            value,
                        },
                    )
                })?;

                res.push(SensorValue { label, value });
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hwmon_metric_entry_try_from_test() -> error::HwmonResult<()> {
        let mut entries = tokio::fs::read_dir(
            "../test_data/test-0/sys/bus/pci/devices/0000:ff:00.0/hwmon/hwmon0",
        )
        .await?;

        if let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let res = MetricEntry::try_from(entry)?;
            assert_eq!(
                res.metric_type,
                MetricType {
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
    async fn sensor_fetch_entries_test() -> error::HwmonResult<()> {
        let path = PathBuf::from("../test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon");
        let res = SensorContainer::fetch_entries(path).await?;
        assert_eq!(res.len(), 16);

        let path = PathBuf::from("invalid_path");
        let res = SensorContainer::fetch_entries(path).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn sensor_build_value_map_test() -> error::HwmonResult<()> {
        let input = vec![];
        let output = SensorContainer::build_value_map(input).await;
        assert_eq!(output.len(), 0);

        let input = vec![
            MetricEntry {
                metric_type: MetricType { hwmon_type: HwmonType::Temperature, idx: 1 },
                metric_item: MetricItem {
                    item_name: String::from("label"),
                    path: PathBuf::from("../test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_label")
                }
            },
            MetricEntry {
                metric_type: MetricType { hwmon_type: HwmonType::Temperature, idx: 1 },
                metric_item: MetricItem {
                    item_name: String::from("input"),
                    path: PathBuf::from("../test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_input")
                }
            },
        ];
        let output = SensorContainer::build_value_map(input).await;
        assert_eq!(output.len(), 1);
        let opt = output.get(&HwmonType::Temperature);
        assert!(opt.is_some());
        let v = opt.unwrap();
        assert_eq!(v.len(), 1);
        let (label, items) = &v[0];
        assert_eq!(label, "Temp1");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_name, "input");

        let input = vec![MetricEntry {
            metric_type: MetricType {
                hwmon_type: HwmonType::Temperature,
                idx: 1,
            },
            metric_item: MetricItem {
                item_name: String::from("input"),
                path: PathBuf::from(
                    "../test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_input",
                ),
            },
        }];
        let output = SensorContainer::build_value_map(input).await;
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
    async fn sensor_build_label_map_test() -> error::HwmonResult<()> {
        let input = vec![];
        let output = SensorContainer::build_label_map(input).await;
        assert_eq!(output.len(), 0);

        let input = vec![MetricEntry {
            metric_type: MetricType {
                hwmon_type: HwmonType::Temperature,
                idx: 1,
            },
            metric_item: MetricItem {
                item_name: String::from("label"),
                path: PathBuf::from(
                    "../test_data/test-0/sys/bus/pci/devices/0000:6d:00.0/hwmon/hwmon0/temp1_label",
                ),
            },
        }];
        let output = SensorContainer::build_label_map(input).await;

        assert_eq!(output.len(), 1);
        let res = output.get(&MetricType {
            hwmon_type: HwmonType::Temperature,
            idx: 1,
        });
        assert!(res.is_some());
        assert_eq!(res.unwrap(), "Temp1");
        assert!(output
            .get(&MetricType {
                hwmon_type: HwmonType::Temperature,
                idx: 2
            })
            .is_none());
        assert!(output
            .get(&MetricType {
                hwmon_type: HwmonType::Power,
                idx: 1
            })
            .is_none());

        Ok(())
    }

    #[tokio::test]
    async fn fetcher_read_test() -> DeviceResult<()> {
        let fetcher = Fetcher::new("../test_data/test-0/sys", 0, "0000:6d:00.0").await?;

        let currents = fetcher.read_currents().await?;
        assert_eq!(currents.len(), 2);
        assert_eq!(currents[0].label, "Current1");
        assert_eq!(currents[0].value, 1000);
        assert_eq!(currents[1].label, "Current2");
        assert_eq!(currents[1].value, 2000);

        let voltages = fetcher.read_voltages().await?;
        assert_eq!(voltages.len(), 2);
        assert_eq!(voltages[0].label, "Voltage0");
        assert_eq!(voltages[0].value, 1100);
        assert_eq!(voltages[1].label, "Voltage1");
        assert_eq!(voltages[1].value, 1200);

        let powers = fetcher.read_powers_average().await?;
        assert_eq!(powers.len(), 2);
        assert_eq!(powers[0].label, "Power1");
        assert_eq!(powers[0].value, 1111);
        assert_eq!(powers[1].label, "Power2");
        assert_eq!(powers[1].value, 22222);

        let temperature = fetcher.read_temperatures().await?;
        assert_eq!(temperature.len(), 2);
        assert_eq!(temperature[0].label, "Temp1");
        assert_eq!(temperature[0].value, 36000);
        assert_eq!(temperature[1].label, "Temp2");
        assert_eq!(temperature[1].value, 37000);

        Ok(())
    }
}
