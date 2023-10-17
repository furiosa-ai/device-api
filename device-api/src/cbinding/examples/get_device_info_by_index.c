#include <stdio.h>
#include "../include/device.h"

int main()
{
    device_handle handle;
    error_code err;
    err = get_device_by_index(0, &handle);
    if (err != ok) {
        printf("failed to get device with an error code %d\n", err);
        return 1;
    }

    uint8_t index;
    err = get_device_index(handle, &index);
    if (err != ok) {
        printf("failed to get device index with an error code %d\n", err);
        return 1;
    }
    printf("the device index is %d\n", index);

    Arch arch;
    err = get_device_arch(handle, &arch);
    if (err != ok) {
        printf("failed to get device arch with an error code %d\n", err);
        return 1;
    }
    printf("the device arch is %d\n", arch);

    bool liveness;
    err = get_device_liveness(handle, &liveness);
    if (err != ok) {
        printf("failed to get device liveness with an error code %d\n", err);
        return 1;
    }
    printf("the device liveness is %d\n", liveness);

    ErrorStatesKeyValuePair* err_stats_output;
    uint8_t len;
    err = get_device_error_states(handle, &err_stats_output, &len);
    if (err != ok) {
        printf("failed to get device error states with an error code %d\n", err);
        return 1;
    }
    for (int i = 0; i < len; i++) {
        printf("the device error states %s / %d\n", err_stats_output[i].key, err_stats_output[i].value);
    }
    destroy_error_states(err_stats_output, len);

    char* output_ptr;
    err = get_device_pci_bus_number(handle, &output_ptr);
    if (err != ok) {
        printf("failed to get device pci bus number with an error code %d\n", err);
        return 1;
    }
    printf("the pci bus number is %s\n", output_ptr);
    free_string(output_ptr);

    err = get_device_pci_dev_id(handle, &output_ptr);
    if (err != ok) {
        printf("failed to get device pci dev id with an error code %d\n", err);
        return 1;
    }
    printf("the pci dev id is %s\n", output_ptr);
    free_string(output_ptr);

    err = get_device_serial_number(handle, &output_ptr);
    if (err != ok) {
        printf("failed to get device serial number with an error code %d\n", err);
        return 1;
    }
    printf("the serial number is %s\n", output_ptr);
    free_string(output_ptr);

    err = get_device_uuid(handle, &output_ptr);
    if (err != ok) {
        printf("failed to get device uuid with an error code %d\n", err);
        return 1;
    }
    printf("the device uuid is %s\n", output_ptr);
    free_string(output_ptr);

    err = get_device_firmware_version(handle, &output_ptr);
    if (err != ok) {
        printf("failed to get device firmware version with an error code %d\n", err);
        return 1;
    }
    printf("the device firmware version is %s\n", output_ptr);
    free_string(output_ptr);

    err = get_device_driver_version(handle, &output_ptr);
    if (err != ok) {
        printf("failed to get device driver version with an error code %d\n", err);
        return 1;
    }
    printf("the device driver version is %s\n", output_ptr);
    free_string(output_ptr);

    uint32_t heartbeat;
    err = get_device_heartbeat(handle, &heartbeat);
    if (err != ok) {
        printf("failed to get device heartbeat with an error code %d\n", err);
        return 1;
    }
    printf("the device heartbeat is %d\n", heartbeat);

    uint8_t numa_node_id;
    err = get_device_numa_node(handle, &numa_node_id);
    if (err != unsupported_error && err != ok) {
        printf("failed to get device numa node id with an error code %d\n", err);
        return 1;
    }
    if (err != unsupported_error) {
        printf("the device numa node id is %d\n", numa_node_id);
    }

    uint8_t core_num;
    err = get_device_core_num(handle, &core_num);
    if (err != ok) {
        printf("failed to get device core num with an error code %d\n", err);
        return 1;
    }
    printf("the device core num is %d\n", core_num);

    uint8_t* cores_output;
    err = get_device_core_ids(handle, &cores_output, &len);
    if (err != ok) {
        printf("failed to get device core ids with an error code %d\n", err);
        return 1;
    }
    for(int i = 0; i < len; i++) {
        CoreStatus status;
        err = get_device_core_status(handle, cores_output[i], &status);
        if (err != ok) {
            printf("failed to get device core status with an error code %d\n", err);
            return 1;
        }
        printf("the device core id(%d)'s status is %d\n", cores_output[i], status);
        if (status == Occupied) {
            char* fd;
            err = get_device_core_occupied_fd(handle, cores_output[i], &fd);
            if (err != ok) {
                printf("failed to get core occupied fd with an error code %d\n", err);
            }
            printf("the fd %s occupied device core id(%d)\n",fd ,cores_output[i]);
            free_string(fd);
        }
    }
    destroy_device_core_ids(cores_output, len);

    DeviceFile* files_output;
    err = get_device_files(handle, &files_output, &len);
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
    destroy_device_files(files_output, len);

    CoreStatusPair* status_output;
    err = get_device_all_core_status(handle, &status_output, &len);
    if (err != ok) {
        printf("failed to get device core status with an error code %d\n", err);
    }
    for (int i = 0; i < len; i++){
        printf("core index is %d\n", status_output[i].core_index);
        printf("core status is %d\n", status_output[i].status);
    }
    destroy_core_status_pair(status_output, len);
    destroy_device_handle(handle);

    return 0;
}