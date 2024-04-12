#![allow(warnings)]
use std::collections::BTreeMap;

use libc::link;
use strum_macros::AsRefStr;

use crate::topology::hwloc::HwlocTopology;
use crate::topology::hwloc_binding::*;
use crate::topology::LinkType::*;
use crate::{Device, DeviceResult};

mod hwloc;
mod hwloc_binding;

#[derive(AsRefStr, Clone, Copy, Debug, PartialEq)]
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

pub trait HardwareTopologyHint {
    fn get_hw_topology_hint(&self) -> String;
}

pub struct Topology {
    topology_matrix: BTreeMap<(String, String), LinkType>,
}

impl Topology {
    pub fn new(devices: Vec<Device>) -> DeviceResult<Topology> {
        let keys = devices.iter().map(|d| d.busname().unwrap()).collect();
        let topology_provider = DefaultTopologyProvider::new()?;
        let populated_matrix = populate_topology_matrix(topology_provider, keys)?;
        Ok(Self {
            topology_matrix: populated_matrix,
        })
    }

    pub fn get_link_type<T: HardwareTopologyHint>(&self, device1: &T, device2: &T) -> LinkType {
        self.get_link_type_with_bdf(
            device1.get_hw_topology_hint(),
            device2.get_hw_topology_hint(),
        )
    }

    fn get_link_type_with_bdf(&self, dev1_bdf: String, dev2_bdf: String) -> LinkType {
        let key = if dev1_bdf > dev2_bdf {
            (dev2_bdf, dev1_bdf)
        } else {
            (dev1_bdf, dev2_bdf)
        };

        match self.topology_matrix.get(&key) {
            Some(link_type) => *link_type,
            None => LinkTypeUnknown,
        }
    }
}

fn populate_topology_matrix<T: TopologyProvider>(
    topology_provider: T,
    devices: Vec<String>,
) -> DeviceResult<BTreeMap<(String, String), LinkType>> {
    let mut topology_matrix: BTreeMap<(String, String), LinkType> = BTreeMap::new();

    for i in 0..devices.len() {
        for j in 0..devices.len() {
            let dev1_bdf = devices.get(i).unwrap().clone();
            let dev2_bdf = devices.get(j).unwrap().clone();
            let mut link_type: LinkType = LinkTypeUnknown;

            if dev1_bdf == dev2_bdf {
                link_type = LinkTypeSoc
            } else {
                link_type = topology_provider.get_common_ancestor_obj(&dev1_bdf, &dev2_bdf)?;
            }

            let key = if dev1_bdf > dev2_bdf {
                (dev2_bdf, dev1_bdf)
            } else {
                (dev1_bdf, dev2_bdf)
            };

            topology_matrix.insert(key, link_type);
        }
    }

    Ok(topology_matrix)
}

trait TopologyProvider {
    fn get_common_ancestor_obj(&self, dev1_bdf: &str, dev2_bdf: &str) -> DeviceResult<LinkType>;
}

struct DefaultTopologyProvider {
    hwloc: HwlocTopology,
}

impl TopologyProvider for DefaultTopologyProvider {
    fn get_common_ancestor_obj(&self, dev1_bdf: &str, dev2_bdf: &str) -> DeviceResult<LinkType> {
        let mut link = LinkTypeUnknown;

        if dev1_bdf == dev2_bdf {
            link = LinkTypeSoc;
        } else {
            let ancestor = self.hwloc.get_common_ancestor_obj(dev1_bdf, dev2_bdf)?;
            match unsafe { (*ancestor).type_ } {
                hwloc_obj_type_t_HWLOC_OBJ_MACHINE => link = LinkTypeInterconnect,
                hwloc_obj_type_t_HWLOC_OBJ_PACKAGE => link = LinkTypeCPU,
                hwloc_obj_type_t_HWLOC_OBJ_BRIDGE => link = LinkTypeHostBridge,
                _ => link = LinkTypeUnknown,
            }
        }

        Ok(link)
    }
}

impl DefaultTopologyProvider {
    fn new() -> DeviceResult<Self> {
        let mut hwloc = HwlocTopology::new();

        // Initialize hwloc topology
        hwloc.init_topology()?;

        // Set I/O types filter
        hwloc.set_io_types_filter(hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_IMPORTANT)?;

        // Load the topology
        hwloc.load_topology()?;

        Ok(Self { hwloc })
    }
}

impl Drop for DefaultTopologyProvider {
    fn drop(&mut self) {
        self.hwloc.destroy_topology();
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::topology::hwloc::HwlocTopology;
    use crate::topology::hwloc_binding::*;
    use crate::topology::LinkType::*;
    use crate::topology::{
        populate_topology_matrix, DefaultTopologyProvider, LinkType, Topology, TopologyProvider,
    };
    use crate::{Device, DeviceResult};

    struct MockTopologyProvider {
        provider: DefaultTopologyProvider,
    }

    impl MockTopologyProvider {
        fn new() -> DeviceResult<Self> {
            let mut hwloc = HwlocTopology::new();
            hwloc.init_topology()?;
            hwloc.set_io_types_filter(hwloc_type_filter_e_HWLOC_TYPE_FILTER_KEEP_IMPORTANT)?;

            let current_dir = env::current_dir().unwrap();
            let xml_path = current_dir.join("src/topology/test.xml");
            hwloc.set_topology_from_xml(xml_path.to_str().unwrap())?;
            hwloc.load_topology()?;

            Ok(Self {
                provider: DefaultTopologyProvider { hwloc },
            })
        }
    }

    impl TopologyProvider for MockTopologyProvider {
        fn get_common_ancestor_obj(
            &self,
            dev1_bdf: &str,
            dev2_bdf: &str,
        ) -> DeviceResult<LinkType> {
            self.provider.get_common_ancestor_obj(dev1_bdf, dev2_bdf)
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

        let populated_matrix =
            populate_topology_matrix(MockTopologyProvider::new().unwrap(), keys).unwrap();

        let mut mock_topology = Topology {
            topology_matrix: populated_matrix,
        };

        assert_eq!(
            mock_topology.get_link_type_with_bdf(String::from(""), String::from("")),
            LinkTypeUnknown
        );
        assert_eq!(
            mock_topology.get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("")),
            LinkTypeUnknown
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:27:00.0")),
            LinkTypeSoc
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:2a:00.0")),
            LinkTypeHostBridge
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:51:00.0")),
            LinkTypeCPU
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:57:00.0")),
            LinkTypeCPU
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:9e:00.0")),
            LinkTypeInterconnect
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:a4:00.0")),
            LinkTypeInterconnect
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:c7:00.0")),
            LinkTypeInterconnect
        );
        assert_eq!(
            mock_topology
                .get_link_type_with_bdf(String::from("0000:27:00.0"), String::from("0000:ca:00.0")),
            LinkTypeInterconnect
        );
    }
}
