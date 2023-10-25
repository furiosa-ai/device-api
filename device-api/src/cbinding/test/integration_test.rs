#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};
    use std::ptr;
    use std::string::String;

    use libc::c_char;

    use crate::cbinding;

    fn setup() {
        std::env::set_var("FURIOSA_DEV_FS", "../test_data/test-0/dev");
        std::env::set_var("FURIOSA_SYS_FS", "../test_data/test-0/sys");
    }

    fn teardown() {
        std::env::remove_var("FURIOSA_DEV_FS");
        std::env::remove_var("FURIOSA_SYS_FS");
    }

    #[test]
    fn test_list_all_devices() {
        setup();

        unsafe {
            let mut device_handles_output: *mut cbinding::device_handle = std::ptr::null_mut();
            let mut output_len: u8 = 0;

            let ret = cbinding::furiosa_device_list(&mut device_handles_output, &mut output_len);
            assert!(matches!(ret, cbinding::error_code::ok));
            assert_eq!(output_len, 2);

            cbinding::furiosa_device_handle_list_destroy(device_handles_output, output_len);
        }

        teardown();
    }

    #[test]
    fn test_get_device_by_index() {
        setup();

        unsafe {
            let mut device_handle_output: cbinding::device_handle = std::ptr::null_mut();

            let mut ret = cbinding::furiosa_device_get_by_index(0_u8, &mut device_handle_output);
            assert!(matches!(ret, cbinding::error_code::ok));

            let mut index_output: u8 = 0;
            ret =
                cbinding::device::furiosa_device_index_get(device_handle_output, &mut index_output);
            assert!(matches!(ret, cbinding::error_code::ok));
            assert_eq!(index_output, 0);

            cbinding::furiosa_device_handle_destroy(device_handle_output);
        }

        unsafe {
            let mut device_handle_output: cbinding::device_handle = std::ptr::null_mut();

            let mut ret = cbinding::furiosa_device_get_by_index(1_u8, &mut device_handle_output);
            assert!(matches!(ret, cbinding::error_code::ok));

            let mut index_output: u8 = 0;
            ret =
                cbinding::device::furiosa_device_index_get(device_handle_output, &mut index_output);
            assert!(matches!(ret, cbinding::error_code::ok));
            assert_eq!(index_output, 1);

            cbinding::furiosa_device_handle_destroy(device_handle_output);
        }

        teardown();
    }

    #[test]
    fn test_get_device_file_by_name() {
        setup();

        unsafe {
            let mut device_file_output: *mut cbinding::device::DeviceFile = std::ptr::null_mut();
            let device_name: *const c_char = CString::new("npu0").unwrap().into_raw();
            let ret =
                cbinding::furiosa_device_get_by_filename(device_name, &mut device_file_output);
            assert!(matches!(ret, cbinding::error_code::ok));

            let path =
                String::from_utf8_lossy(CStr::from_ptr((*device_file_output).path).to_bytes())
                    .to_string();
            assert!(path.contains("npu0"));

            cbinding::furiosa_device_file_destroy(device_file_output);
        }

        unsafe {
            let mut device_file_output: *mut cbinding::device::DeviceFile = std::ptr::null_mut();
            let device_name: *const c_char = CString::new("npu0pe0").unwrap().into_raw();
            let ret =
                cbinding::furiosa_device_get_by_filename(device_name, &mut device_file_output);
            assert!(matches!(ret, cbinding::error_code::ok));

            let path =
                String::from_utf8_lossy(CStr::from_ptr((*device_file_output).path).to_bytes())
                    .to_string();
            assert!(path.contains("npu0pe0"));

            cbinding::furiosa_device_file_destroy(device_file_output);
        }

        unsafe {
            let mut device_file_output: *mut cbinding::device::DeviceFile = std::ptr::null_mut();
            let device_name: *const c_char = CString::new("npu0pe1").unwrap().into_raw();
            let ret =
                cbinding::furiosa_device_get_by_filename(device_name, &mut device_file_output);
            assert!(matches!(ret, cbinding::error_code::ok));

            let path =
                String::from_utf8_lossy(CStr::from_ptr((*device_file_output).path).to_bytes())
                    .to_string();
            assert!(path.contains("npu0pe1"));

            cbinding::furiosa_device_file_destroy(device_file_output);
        }

        unsafe {
            let mut device_file_output: *mut cbinding::device::DeviceFile = std::ptr::null_mut();
            let device_name: *const c_char = CString::new("npu0pe0-1").unwrap().into_raw();
            let ret =
                cbinding::furiosa_device_get_by_filename(device_name, &mut device_file_output);
            assert!(matches!(ret, cbinding::error_code::ok));

            let path =
                String::from_utf8_lossy(CStr::from_ptr((*device_file_output).path).to_bytes())
                    .to_string();
            assert!(path.contains("npu0pe0-1"));

            cbinding::furiosa_device_file_destroy(device_file_output);
        }

        teardown();
    }

    #[test]
    fn test_device_apis() {
        setup();

        unsafe {
            let mut device_handles_output: *mut cbinding::device_handle = std::ptr::null_mut();
            let mut output_len: u8 = 0;

            let mut ret =
                cbinding::furiosa_device_list(&mut device_handles_output, &mut output_len);
            assert!(matches!(ret, cbinding::error_code::ok));
            assert_eq!(output_len, 2);

            for device_handle_idx in 0..output_len {
                let device_handle = device_handles_output.offset(device_handle_idx as isize);
                let mut device_index_output: u8 = 0;
                ret = cbinding::device::furiosa_device_index_get(
                    *device_handle,
                    &mut device_index_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(device_index_output, device_handle_idx);

                let mut arch_output = cbinding::arch::Arch::warboy_a0;
                ret = cbinding::device::furiosa_device_arch_get(*device_handle, &mut arch_output);
                assert!(matches!(ret, cbinding::error_code::ok));
                assert!(matches!(arch_output, cbinding::arch::Arch::warboy));

                let mut liveness_output = false;
                ret = cbinding::device::furiosa_device_liveness_get(
                    *device_handle,
                    &mut liveness_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert!(liveness_output);

                let mut err_stats_output: *mut cbinding::device::ErrorStatesKeyValuePair =
                    ptr::null_mut();
                let mut err_state_len: u8 = 0;
                ret = cbinding::device::furiosa_device_error_states_get(
                    *device_handle,
                    &mut err_stats_output,
                    &mut err_state_len,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(err_state_len, 9);
                for idx in 0..err_state_len {
                    let key_value_pair = err_stats_output.offset(idx as isize);
                    let key =
                        String::from_utf8_lossy(CStr::from_ptr((*key_value_pair).key).to_bytes())
                            .to_string();
                    match key.as_str() {
                        "axi_post_error" => {}
                        "axi_fetch_error" => {}
                        "axi_discard_error" => {}
                        "axi_doorbell_done" => {}
                        "pcie_post_error" => {}
                        "pcie_fetch_error" => {}
                        "pcie_discard_error" => {}
                        "pcie_doorbell_done" => {}
                        "device_error" => {}
                        _ => panic!(),
                    }
                }
                cbinding::device::furiosa_error_states_destroy(err_stats_output, err_state_len);

                let mut pci_bus_num_output: *mut c_char = ptr::null_mut();
                ret = cbinding::device::furiosa_device_pci_bus_number_get(
                    *device_handle,
                    &mut pci_bus_num_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                let bus_num =
                    String::from_utf8_lossy(CStr::from_ptr(pci_bus_num_output).to_bytes())
                        .to_string();
                assert!(bus_num == "0000:6d:00.0" || bus_num == "0000:ff:00.0");
                cbinding::device::furiosa_string_free(pci_bus_num_output);

                let mut pci_dev_id_output: *mut c_char = ptr::null_mut();
                ret = cbinding::device::furiosa_device_pci_dev_id_get(
                    *device_handle,
                    &mut pci_dev_id_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                let pci_dev_id =
                    String::from_utf8_lossy(CStr::from_ptr(pci_dev_id_output).to_bytes())
                        .to_string();
                assert!(pci_dev_id == "234:0" || pci_dev_id == "510:0");
                cbinding::device::furiosa_string_free(pci_dev_id_output);

                let mut serial_number_output: *mut c_char = ptr::null_mut();
                ret = cbinding::device::furiosa_device_serial_number_get(
                    *device_handle,
                    &mut serial_number_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                let serial_number =
                    String::from_utf8_lossy(CStr::from_ptr(serial_number_output).to_bytes())
                        .to_string();
                assert!(
                    serial_number == "WBYB0236FH505KREO" || serial_number == "WBYB0236FH543KREO"
                );
                cbinding::device::furiosa_string_free(serial_number_output);

                let mut device_uuid_output: *mut c_char = ptr::null_mut();
                ret = cbinding::device::furiosa_device_uuid_get(
                    *device_handle,
                    &mut device_uuid_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                let device_uuid =
                    String::from_utf8_lossy(CStr::from_ptr(device_uuid_output).to_bytes())
                        .to_string();
                assert!(
                    device_uuid == "A76AAD68-6855-40B1-9E86-D080852D1C84"
                        || device_uuid == "A76AAD68-96A2-4B6A-A879-F91B8224DC84"
                );
                cbinding::device::furiosa_string_free(device_uuid_output);

                let mut device_firmware_version_output: *mut c_char = ptr::null_mut();
                ret = cbinding::device::furiosa_device_firmware_version_get(
                    *device_handle,
                    &mut device_firmware_version_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                let firmware_version = String::from_utf8_lossy(
                    CStr::from_ptr(device_firmware_version_output).to_bytes(),
                )
                .to_string();
                assert_eq!(firmware_version, "1.6.0, c1bebfd");
                cbinding::device::furiosa_string_free(device_firmware_version_output);

                let mut device_driver_version_output: *mut c_char = ptr::null_mut();
                ret = cbinding::device::furiosa_device_driver_version_get(
                    *device_handle,
                    &mut device_driver_version_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                let driver_version = String::from_utf8_lossy(
                    CStr::from_ptr(device_driver_version_output).to_bytes(),
                )
                .to_string();
                assert_eq!(driver_version, "1.9.2, 3def9c2");
                cbinding::device::furiosa_string_free(device_driver_version_output);

                let mut heartbeat_output: u32 = 0;
                ret = cbinding::device::furiosa_device_heartbeat_get(
                    *device_handle,
                    &mut heartbeat_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(heartbeat_output, 14649);

                let mut numa_node_output: u8 = 0;
                ret = cbinding::device::furiosa_device_numa_node_get(
                    *device_handle,
                    &mut numa_node_output,
                );
                assert!(matches!(
                    ret,
                    cbinding::error_code::ok | cbinding::error_code::unsupported_error
                ));

                let mut core_num_output: u8 = 0;
                ret = cbinding::device::furiosa_device_core_num_get(
                    *device_handle,
                    &mut core_num_output,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(core_num_output, 2);

                let mut core_ids_output: *mut u8 = ptr::null_mut();
                let mut core_ids_len: u8 = 0;
                ret = cbinding::device::furiosa_device_core_ids_get(
                    *device_handle,
                    &mut core_ids_output,
                    &mut core_ids_len,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(core_ids_len, 2);
                for idx in 0..core_ids_len {
                    let id = *(core_ids_output.offset(idx as isize));
                    assert_eq!(id, idx);
                    let mut core_status_output: cbinding::device::CoreStatus =
                        cbinding::device::CoreStatus::unavailable;
                    ret = cbinding::device::furiosa_device_core_status_get(
                        *device_handle,
                        id,
                        &mut core_status_output,
                    );
                    assert!(matches!(ret, cbinding::error_code::ok));
                    assert!(matches!(
                        core_status_output,
                        cbinding::device::CoreStatus::available
                    ));
                    //@bg: no way to test "get_device_core_occupied_fd" since mock fd can be opened multiple times.
                }
                cbinding::device::furiosa_device_core_ids_destroy(core_ids_output, core_ids_len);

                let mut device_files_output: *mut cbinding::device::DeviceFile = ptr::null_mut();
                let mut device_files_len: u8 = 0;
                cbinding::device::furiosa_device_file_list(
                    *device_handle,
                    &mut device_files_output,
                    &mut device_files_len,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(device_files_len, 4);
                for idx in 0..device_files_len {
                    let device_file = device_files_output.offset(idx as isize);
                    assert_eq!((*device_file).device_index, device_handle_idx);
                }

                let mut all_core_status_output: *mut cbinding::device::CoreStatusPair =
                    ptr::null_mut();
                let mut all_core_status_len: u8 = 0;
                ret = cbinding::device::furiosa_device_all_core_status_get(
                    *device_handle,
                    &mut all_core_status_output,
                    &mut all_core_status_len,
                );
                assert!(matches!(ret, cbinding::error_code::ok));
                assert_eq!(all_core_status_len, 2);
                for idx in 0..all_core_status_len {
                    let pair = all_core_status_output.offset(idx as isize);
                    assert!((*pair).core_index < all_core_status_len);
                }
                cbinding::device::furiosa_core_status_pair_destroy(
                    all_core_status_output,
                    all_core_status_len,
                );
            }

            cbinding::furiosa_device_handle_list_destroy(device_handles_output, output_len);
        }

        teardown();
    }
}
