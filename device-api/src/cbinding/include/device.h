#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Enum for the NPU architecture.
 */
typedef enum Arch {
  warboy_a0,
  warboy,
  renegade,
  u250,
} Arch;

/**
 * \brief Represent a core range type
 */
typedef enum CoreRangeType {
  all,
  range,
} CoreRangeType;

/**
 * \brief Represent a core status
 */
typedef enum CoreStatus {
  available,
  occupied,
  unavailable,
} CoreStatus;

/**
 * \brief Represent a device mode
 */
typedef enum DeviceMode {
  single,
  fusion,
  multi_core,
} DeviceMode;

/**
 * \brief Represent a return status
 */
typedef enum error_code {
  /**
   * When a function call is successful.
   */
  ok = 0,
  /**
   * When a input is invalid.
   */
  invalid_input,
  /**
   * When a function call fails while constructing c string from invalid bytes containing the null byte.
   */
  null_error,
  /**
   * When a certain operation is not supported on the system.
   */
  unsupported_error,
  /**
   * When a certain operation is not available in the current situation.
   */
  unavailable_error,
  /**
   * When a device is not found with the given option.
   */
  device_not_found,
  /**
   * When a device state is busy.
   */
  device_busy,
  /**
   * When a certain operation failed by an unexpected io error.
   */
  io_error,
  /**
   * When a certain operation failed by a permission deny.
   */
  permission_denied_error,
  /**
   * When an arch is unknown.
   */
  unknown_arch_error,
  /**
   * When a driver is incompatible.
   */
  incompatible_driver_error,
  /**
   * When hwmon error is occurred
   */
  hwmon_error,
  /**
   * When performance counter error is occurred
   */
  performance_counter_error,
  /**
   * When a retrieved value is invalid.
   */
  unexpected_value_error,
  /**
   * When a unicode parsing is failed
   */
  parse_error,
  /**
   * When a reason is unknown
   */
  unknown_error,
} error_code;

/**
 * \brief Opaque type for Device
 */
typedef struct DeviceHandle DeviceHandle;

/**
 * \brief Handle of Device
 */
typedef struct DeviceHandle *device_handle;

/**
 * \brief Represent a core range
 */
typedef struct CoreRange {
  enum CoreRangeType range_type;
  uint8_t start;
  uint8_t end;
} CoreRange;

/**
 * \brief Output of `furiosa_device_file_list`
 */
typedef struct DeviceFile {
  uint8_t device_index;
  struct CoreRange core_range;
  const char *path;
  enum DeviceMode mode;
} DeviceFile;

/**
 * \brief Output of `furiosa_device_error_states_get`
 */
typedef struct ErrorStatesKeyValuePair {
  const char *key;
  uint32_t value;
} ErrorStatesKeyValuePair;

/**
 * \brief Output of `furiosa_device_all_core_status_get`
 */
typedef struct CoreStatusPair {
  uint8_t core_index;
  enum CoreStatus status;
} CoreStatusPair;

/**
 * \brief Retrieve device_handle of all Furiosa NPU devices in the system.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved device_handles must be destroyed by `furiosa_device_handle_list_destroy`.
 *
 * @param[out] output output buffer for array of device_handle.
 * @param[out] output_len output buffer for length of array.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_list(device_handle **output, uint8_t *output_len);

/**
 * \brief Destroy array of device_handle returned by `furiosa_device_list`.
 *
 * @param raw pointer to array of device_handles.
 * @param len length of array.
 */
void furiosa_device_handle_list_destroy(device_handle *raw, uint8_t len);

/**
 * \brief Retrieve device_handle with a specific index of Furiosa NPU device in the system.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved device_handle must be destroyed by `furiosa_device_handle_destroy`.
 *
 * @param idx index of Furiosa NPU device.
 * @param[out] output output buffer for device_handle.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_get_by_index(uint8_t idx, device_handle *output);

/**
 * \brief Destroy device_handle returned by `furiosa_device_get_by_index`.
 *
 * @param device device_handle to destroy.
 */
void furiosa_device_handle_destroy(device_handle device);

/**
 * \brief Retrieve DeviceFile with a specific name of Furiosa NPU device in the system.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved DeviceFile must be destroyed by `furiosa_device_file_destroy`.
 *
 * @parm device_name pointer to C string for a device name (e.g., npu0, npu0pe0, npu0pe0-1),
 * the name should be terminated by null character.
 * @param[out] output output buffer for DeviceFile.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_get_by_filename(const char *device_name, struct DeviceFile **output);

/**
 * \brief Destroy DeviceFile returned by `furiosa_device_get_by_filename`.
 *
 * @param raw pointer to `DeviceFile` to destroy.
 */
void furiosa_device_file_destroy(struct DeviceFile *raw);

/**
 * \brief Retrieve the name of the device (e.g., npu0).
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for pointer to C string.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_name_get(device_handle handle, char **output);

/**
 * \brief Retrieve the device index (e.g., 0 for npu0).
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for index of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_index_get(device_handle handle, uint8_t *output);

/**
 * \brief Retrieve `Arch` of the device(e.g., `warboy`).
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for arch of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_arch_get(device_handle handle, enum Arch *output);

/**
 * \brief Retrieve a liveness state of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for liveness of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_liveness_get(device_handle handle, bool *output);

/**
 * \brief Retrieve error states of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved array of ErrorStatesKeyValuePair must be destroyed by `furiosa_error_states_destroy`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for array of `ErrorStatesKeyValuePair`.
 * @param[out] output_len output buffer for length of array.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_error_states_get(device_handle handle,
                                                struct ErrorStatesKeyValuePair **output,
                                                uint8_t *output_len);

/**
 * \brief Safely free device error states array of `ErrorStatesKeyValuePair` allocated by `furiosa_device_error_states_get`.
 *
 * @param raw pointer to array of `ErrorStatesKeyValuePair`.
 * @param len length of array.
 */
void furiosa_error_states_destroy(struct ErrorStatesKeyValuePair *raw,
                                  uint8_t len);

/**
 * \brief Retrieve PCI bus number of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for PCI bus number of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_pci_bus_number_get(device_handle handle, char **output);

/**
 * \brief Retrieve PCI device ID of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for PCI bus number of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_pci_dev_id_get(device_handle handle, char **output);

/**
 * \brief Retrieve serial number of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for serial number of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_serial_number_get(device_handle handle, char **output);

/**
 * \brief Retrieve UUID of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for UUID of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_uuid_get(device_handle handle, char **output);

/**
 * \brief Retrieves firmware revision from the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for firmware revision of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_firmware_version_get(device_handle handle, char **output);

/**
 * \brief Retrieves driver version for the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for driver revision of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_driver_version_get(device_handle handle, char **output);

/**
 * \brief Retrieves uptime of the device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for driver revision of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_heartbeat_get(device_handle handle, uint32_t *output);

/**
 * \brief Retrieve NUMA node ID associated with the NPU's PCI lane
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for NUMA node ID of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_numa_node_get(device_handle handle, uint8_t *output);

/**
 * \brief Retrieve the number of cores
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for the number of cores of device.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_core_num_get(device_handle handle, uint8_t *output);

/**
 * \brief Retrieve the core indices
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved array of core id must be destroyed by `furiosa_device_core_ids_destroy`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for array of core id.
 * @param[out] output_len output buffer for length of array.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_core_ids_get(device_handle handle,
                                            uint8_t **output,
                                            uint8_t *output_len);

/**
 * \brief Safely free the array of device core id that is allocated by `furiosa_device_core_ids_get`.
 *
 * @param raw pointer to array of device core id.
 * @param len length of array.
 */
void furiosa_device_core_ids_destroy(uint8_t *raw,
                                     uint8_t len);

/**
 * \brief Retrieve the list device files under the given device.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved array of DeviceFile must be destroyed by `furiosa_device_file_list_destroy`.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for array of DeviceFile.
 * @param[out] output_len output buffer for length of array.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_file_list(device_handle handle,
                                         struct DeviceFile **output,
                                         uint8_t *output_len);

/**
 * \brief Safely free the array of DeviceFile that is allocated by `furiosa_device_file_list`.
 *
 * @param raw pointer to array of DeviceFile.
 * @param len length of array.
 */
void furiosa_device_file_list_destroy(struct DeviceFile *raw, uint8_t len);

/**
 * \brief Examine a specific core of the device, whether it is available or not.
 *
 * \remark output buffer must be allocated from outside of FFI boundary.
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param core_idx index of a specific core.
 * @param[out] output output buffer for core status.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_core_status_get(device_handle handle,
                                               uint8_t core_idx,
                                               enum CoreStatus *output);

/**
 * \brief Retrieve the file descriptor occupied a specific core.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved C string must be destroyed by `furiosa_string_free`
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param core_idx index of a specific core.
 * @param[out] output output buffer for file descriptor.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_core_occupied_fd_get(device_handle handle,
                                                    uint8_t core_idx,
                                                    char **output);

/**
 * \brief Examine each core of the device, whether it is available or not.
 *
 * \remark output buffer must be allocated from outside of FFI boundary,
 * and retrieved array of `CoreStatusPair` must be destroyed by `furiosa_core_status_pair_destroy`
 *
 * @param handle device_handle of Furiosa NPU device.
 * @param[out] output output buffer for the array of `CoreStatusPair`.
 * @param[out] output_len output buffer for length of array.
 * @return error_code::ok if successful, see `error_code` for error cases.
 */
enum error_code furiosa_device_all_core_status_get(device_handle handle,
                                                   struct CoreStatusPair **output,
                                                   uint8_t *output_len);

/**
 * \brief Safely free array of `CoreStatusPair` that is allocated by `furiosa_device_all_core_status_get`.
 *
 * @param raw pointer to array of `CoreStatusPair`.
 * @param len length of array.
 */
void furiosa_core_status_pair_destroy(struct CoreStatusPair *raw,
                                      uint8_t len);

/**
 * \brief Safely free rust string that is represented in C string.
 *
 * @param ptr pointer to rust string.
 */
void furiosa_string_free(const char *ptr);
