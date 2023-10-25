#include <stdio.h>
#include "../include/device.h"

int main()
{
    device_handle* device_handles;
    uint8_t handles_len;
    error_code err;

    err = furiosa_device_list(&device_handles, &handles_len);
    if (err != ok) {
        printf("failed to list devices with an error code %d\n", err);
        return 1;
    }

    for (int i = 0; i < handles_len; i++) {
        device_handle handle = device_handles[i];
        uint8_t index;
        err = furiosa_device_index_get(handle, &index);
        if (err != ok) {
            printf("failed to get device index with an error code %d\n", err);
            return 1;
        }
        printf("the device index is %d\n", index);

        Arch arch;
        err = furiosa_device_arch_get(handle, &arch);
        if (err != ok) {
            printf("failed to get device arch with an error code %d\n", err);
            return 1;
        }
        printf("the device arch is %d\n", arch);

        bool liveness;
        err = furiosa_device_liveness_get(handle, &liveness);
        if (err != ok) {
            printf("failed to get device liveness with an error code %d\n", err);
            return 1;
        }
        printf("the device liveness is %d\n", liveness);

        ErrorStatesKeyValuePair* err_stats_output;
        uint8_t len;
        err = furiosa_device_error_states_get(handle, &err_stats_output, &len);
        if (err != ok) {
            printf("failed to get device error states with an error code %d\n", err);
            return 1;
        }
        for (int i = 0; i < len; i++) {
            printf("the device error states %s / %d\n", err_stats_output[i].key, err_stats_output[i].value);
        }
        furiosa_error_states_destroy(err_stats_output, len);

        char* output_ptr;
        err = furiosa_device_pci_bus_number_get(handle, &output_ptr);
        if (err != ok) {
            printf("failed to get device pci bus number with an error code %d\n", err);
            return 1;
        }
        printf("the pci bus number is %s\n", output_ptr);
        furiosa_string_free(output_ptr);

        err = furiosa_device_pci_dev_id_get(handle, &output_ptr);
        if (err != ok) {
            printf("failed to get device pci dev id with an error code %d\n", err);
            return 1;
        }
        printf("the pci dev id is %s\n", output_ptr);
        furiosa_string_free(output_ptr);

        err = furiosa_device_serial_number_get(handle, &output_ptr);
        if (err != ok) {
            printf("failed to get device serial number with an error code %d\n", err);
            return 1;
        }
        printf("the serial number is %s\n", output_ptr);
        furiosa_string_free(output_ptr);

        err = furiosa_device_uuid_get(handle, &output_ptr);
        if (err != ok) {
            printf("failed to get device uuid with an error code %d\n", err);
            return 1;
        }
        printf("the device uuid is %s\n", output_ptr);
        furiosa_string_free(output_ptr);

        err = furiosa_device_firmware_version_get(handle, &output_ptr);
        if (err != ok) {
            printf("failed to get device firmware version with an error code %d\n", err);
            return 1;
        }
        printf("the device firmware version is %s\n", output_ptr);
        furiosa_string_free(output_ptr);

        err = furiosa_device_driver_version_get(handle, &output_ptr);
        if (err != ok) {
            printf("failed to get device driver version with an error code %d\n", err);
            return 1;
        }
        printf("the device driver version is %s\n", output_ptr);
        furiosa_string_free(output_ptr);

        uint32_t heartbeat;
        err = furiosa_device_heartbeat_get(handle, &heartbeat);
        if (err != ok) {
            printf("failed to get device heartbeat with an error code %d\n", err);
            return 1;
        }
        printf("the device heartbeat is %d\n", heartbeat);

        uint8_t numa_node_id;
        err = furiosa_device_numa_node_get(handle, &numa_node_id);
        if (err != unsupported_error && err != ok) {
            printf("failed to get device numa node id with an error code %d\n", err);
            return 1;
        }
        if (err != unsupported_error) {
            printf("the device numa node id is %d\n", numa_node_id);
        }

        uint8_t core_num;
        err = furiosa_device_core_num_get(handle, &core_num);
        if (err != ok) {
            printf("failed to get device core num with an error code %d\n", err);
            return 1;
        }
        printf("the device core num is %d\n", core_num);

        uint8_t* cores_output;
        err = furiosa_device_core_ids_get(handle, &cores_output, &len);
        if (err != ok) {
            printf("failed to get device core ids with an error code %d\n", err);
            return 1;
        }

        for(int i = 0; i < len; i++) {
            CoreStatus status;
            err = furiosa_device_core_status_get(handle, cores_output[i], &status);
            if (err != ok) {
                printf("failed to get device core status with an error code %d\n", err);
                return 1;
            }
            printf("the device core id(%d)'s status is %d\n", cores_output[i], status);

            if (status == occupied) {
                char* fd;
                err = furiosa_device_core_occupied_fd_get(handle, cores_output[i], &fd);
                if (err != ok) {
                    printf("failed to get core occupied fd with an error code %d\n", err);
                }
                printf("the fd %s occupied device core id(%d)\n",fd ,cores_output[i]);
                furiosa_string_free(fd);
            }
        }
        furiosa_device_core_ids_destroy(cores_output, len);

        DeviceFile* files_output;
        err = furiosa_device_file_list(handle, &files_output, &len);
        if (err != ok) {
            printf("failed to get device files with an error code %d\n", err);
        }
        for(int i = 0; i < len; i++) {
            printf("device index is %d\n", files_output[i].device_index);
            printf("device core range type is %d\n", files_output[i].core_range.range_type);
            printf("device core range start is %d\n", files_output[i].core_range.start);
            printf("device core range end is %d\n", files_output[i].core_range.end);
            printf("device path is %s\n", files_output[i].path);
            printf("device mode is %d\n", files_output[i].mode);
        }
        furiosa_device_file_list_destroy(files_output, len);


        CoreStatusPair* status_output;
        err = furiosa_device_all_core_status_get(handle, &status_output, &len);
        if (err != ok) {
            printf("failed to get device core status with an error code %d\n", err);
        }
        for (int i = 0; i < len; i++){
            printf("core index is %d\n", status_output[i].core_index);
            printf("core status is %d\n", status_output[i].status);
        }
        furiosa_core_status_pair_destroy(status_output, len);
    }
    furiosa_device_handle_list_destroy(device_handles, handles_len);

    return 0;
}