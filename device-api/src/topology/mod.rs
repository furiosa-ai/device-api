#![allow(warnings)]
use std::collections::BTreeMap;

use crate::topology::bindgen::*;
use crate::topology::hwloc::{Hwloc, HwlocTopology};
use crate::topology::LinkType::*;
use crate::{Device, DeviceResult};

mod bindgen;
mod build;
mod helper;
mod hwloc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinkType {
    // LinkTypeUnknown unknown
    LinkTypeUnknown = 0,
    // LinkTypeInterconnect two devices are connected across different cpus through interconnect.
    LinkTypeInterconnect = 10,
    // LinkTypeCPU two devices are connected under the same cpu, it may mean:
    // devices are directly attached to the cpu pcie lane without PCIE switch.
    // devices are attached to different PCIE switches under the same cpu.
    LinkTypeCPU = 20,
    // LinkTypeHostBridge two devices are connected under the same PCIE host bridge.
    // Note that this does not guarantee devices are attached to the same PCIE switch.
    // More switches could exist under the host bridge switch.
    LinkTypeHostBridge = 30,

    // NOTE(@bg): Score 40 and 50 is reserved for LinkTypeMultiSwitch and LinkTypeSingleSwitch.
    // NOTE(@bg): Score 60 is reserved for LinkTypeBoard

    // LinkTypeSoc two devices are on the same Soc chip.
    LinkTypeSoc = 70,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkTypeUnknown => "LinkTypeUnknown",
            LinkTypeInterconnect => "LinkTypeInterconnect",
            LinkTypeCPU => "LinkTypeCPU",
            LinkTypeHostBridge => "LinkTypeHostBridge",
            LinkTypeSoc => "LinkTypeSoc",
        }
    }
}

pub struct Topology {
    hwloc_topology: Box<dyn Hwloc>,
    topology_matrix: BTreeMap<(String, String), LinkType>,
}

impl Topology {
    pub fn new() -> Topology {
        Self {
            hwloc_topology: Box::new(HwlocTopology::new()),
            topology_matrix: BTreeMap::new(),
        }
    }
    pub unsafe fn populate(&mut self, devices: Vec<Device>) -> DeviceResult<()> {
        let keys = devices.iter().map(|d| d.busname().unwrap()).collect();
        self.populate_with_keys(keys)
    }

    unsafe fn populate_with_keys(&mut self, devices: Vec<String>) -> DeviceResult<()> {
        // Initialize hwloc topology
        self.hwloc_topology.init_topology()?;

        // Set I/O types filter
        self.hwloc_topology
            .set_io_types_filter(hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_IMPORTANT)?;

        // Load the topology
        self.hwloc_topology.load_topology()?;

        match self.populate_topology_matrix(devices) {
            Ok(matrix) => Ok(()),
            Err(e) => Err(e),
        }
    }

    unsafe fn populate_topology_matrix(&mut self, devices: Vec<String>) -> DeviceResult<()> {
        for i in 0..devices.len() {
            for j in 0..devices.len() {
                let dev1_bdf = devices.get(i).unwrap().clone();
                let dev2_bdf = devices.get(j).unwrap().clone();

                let link_type = self.search_interconnect(&dev1_bdf, &dev2_bdf);
                let key = if dev1_bdf > dev2_bdf {
                    (dev2_bdf, dev1_bdf)
                } else {
                    (dev1_bdf, dev2_bdf)
                };

                self.topology_matrix.insert(key, link_type);
            }
        }

        Ok(())
    }

    unsafe fn search_interconnect(&self, dev1_bdf: &str, dev2_bdf: &str) -> LinkType {
        if dev1_bdf == dev2_bdf {
            return LinkTypeSoc;
        }

        let ancestor_obj = self
            .hwloc_topology
            .get_common_ancestor_obj(dev1_bdf, dev2_bdf)
            .unwrap();

        match (*ancestor_obj).type_ {
            hwloc_obj_type_t_HWLOC_OBJ_MACHINE => LinkTypeInterconnect,
            hwloc_obj_type_t_HWLOC_OBJ_PACKAGE => LinkTypeCPU,
            hwloc_obj_type_t_HWLOC_OBJ_BRIDGE => LinkTypeHostBridge,
            _ => LinkTypeUnknown,
        }
    }

    pub fn get_link_type(&self, dev1_bdf: &str, dev2_bdf: &str) -> LinkType {
        let key = if dev1_bdf > dev2_bdf {
            (dev2_bdf.to_string(), dev1_bdf.to_string())
        } else {
            (dev1_bdf.to_string(), dev2_bdf.to_string())
        };

        match self.topology_matrix.get(&key) {
            Some(link_type) => *link_type,
            None => LinkTypeUnknown,
        }
    }
}

impl Drop for Topology {
    fn drop(&mut self) {
        unsafe { self.hwloc_topology.destroy_topology() }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;

    use crate::topology::bindgen::*;
    use crate::topology::hwloc::{Hwloc, HwlocTopology};
    use crate::topology::LinkType::*;
    use crate::topology::{LinkType, Topology};
    use crate::{Device, DeviceResult};

    struct HwlocTopologyMock {
        hwloc_topology: HwlocTopology,
    }

    impl HwlocTopologyMock {
        pub fn new() -> Self {
            Self {
                hwloc_topology: HwlocTopology::new(),
            }
        }
    }

    impl Hwloc for HwlocTopologyMock {
        unsafe fn init_topology(&mut self) -> DeviceResult<()> {
            self.hwloc_topology.init_topology()
        }

        unsafe fn set_io_types_filter(&mut self, filter: hwloc_type_filter_e) -> DeviceResult<()> {
            self.hwloc_topology.set_io_types_filter(filter)
        }

        unsafe fn load_topology(&mut self) -> DeviceResult<()> {
            let current_dir = env::current_dir().unwrap();
            let xml_path = current_dir.join("src/topology/test.xml");
            self.hwloc_topology
                .set_topology_from_xml(xml_path.to_str().unwrap())?;
            self.hwloc_topology.load_topology()
        }

        unsafe fn set_topology_from_xml(&mut self, xml_path: &str) -> DeviceResult<()> {
            self.hwloc_topology.set_topology_from_xml(xml_path)
        }

        unsafe fn get_common_ancestor_obj(
            &self,
            dev1bdf: &str,
            dev2bdf: &str,
        ) -> DeviceResult<hwloc_obj_t> {
            self.hwloc_topology
                .get_common_ancestor_obj(dev1bdf, dev2bdf)
        }

        unsafe fn destroy_topology(&mut self) {
            self.hwloc_topology.destroy_topology()
        }
    }

    #[test]
    fn test_hwloc_init_and_destroy() {
        let current_dir = env::current_dir().unwrap();
        let xml_path = current_dir.join("src/topology/test.xml");

        let mut hwloc_topology = HwlocTopology::new();
        unsafe {
            assert!(hwloc_topology.init_topology().is_ok());

            assert!(hwloc_topology
                .set_io_types_filter(hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_IMPORTANT)
                .is_ok());

            assert!(hwloc_topology
                .set_topology_from_xml(xml_path.to_str().unwrap())
                .is_ok());

            assert!(hwloc_topology.load_topology().is_ok());
            hwloc_topology.destroy_topology()
        }
    }

    // below hardware topology is used for testing
    // Machine
    // ├── Package (CPU)
    // │   ├── Host Bridge (Root Complex)
    // │   │   └── PCI Bridge
    // │   │       └── PCI Bridge
    // │   │           └── PCI Bridge
    // │   │               └── PCI Bridge
    // │   │                   └── PCI Bridge
    // │   │                       └── PCI Bridge
    // │   │                           └── PCI Bridge
    // │   │                               ├── NPU0(0000:27:00.0)
    // │   │                               └── NPU1(0000:2a:00.0)
    // │   └── Host Bridge (Root Complex)
    // │       └── PCI Bridge
    // │           └── PCI Bridge
    // │               └── PCI Bridge
    // │                   └── PCI Bridge
    // │                       └── PCI Bridge
    // │                           └── PCI Bridge
    // │                               └── PCI Bridge
    // │                                   ├── NPU2(0000:51:00.0)
    // │                                   └── NPU3(0000:57:00.0)
    // └── Package (CPU)
    //     ├── Host Bridge (Root Complex)
    //     │   └── PCI Bridge
    //     │       └── PCI Bridge
    //     │           └── PCI Bridge
    //     │               └── PCI Bridge
    //     │                   └── PCI Bridge
    //     │                       └── PCI Bridge
    //     │                           └── PCI Bridge
    //     │                               ├── NPU4(0000:9e:00.0)
    //     │                               └── NPU5(0000:a4:00.0)
    //     └── Host Bridge (Root Complex)
    //         └── PCI Bridge
    //             └── PCI Bridge
    //                 └── PCI Bridge
    //                     └── PCI Bridge
    //                         └── PCI Bridge
    //                             └── PCI Bridge
    //                                 └── PCI Bridge
    //                                     ├── NPU6(0000:c7:00.0)
    //                                     └── NPU7(0000:ca:00.0)
    #[test]
    fn test_topology_get_link_type() {
        let devices: Vec<Device> = vec![];

        let mut mock_topology = Topology {
            hwloc_topology: Box::new(HwlocTopologyMock::new()),
            topology_matrix: BTreeMap::new(),
        };

        let keys = vec![
            "0000:27:00.0".to_string(),
            "0000:2a:00.0".to_string(),
            "0000:51:00.0".to_string(),
            "0000:57:00.0".to_string(),
            "0000:9e:00.0".to_string(),
            "0000:a4:00.0".to_string(),
            "0000:c7:00.0".to_string(),
            "0000:ca:00.0".to_string(),
        ];
        unsafe {
            assert!(mock_topology.populate_with_keys(keys).is_ok());
        }

        assert_eq!(mock_topology.get_link_type("", ""), LinkTypeUnknown);
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", ""),
            LinkTypeUnknown
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:27:00.0"),
            LinkTypeSoc
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:2a:00.0"),
            LinkTypeHostBridge
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:51:00.0"),
            LinkTypeCPU
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:57:00.0"),
            LinkTypeCPU
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:9e:00.0"),
            LinkTypeInterconnect
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:a4:00.0"),
            LinkTypeInterconnect
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:c7:00.0"),
            LinkTypeInterconnect
        );
        assert_eq!(
            mock_topology.get_link_type("0000:27:00.0", "0000:ca:00.0"),
            LinkTypeInterconnect
        );
    }
}
