#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum Arch {
  WarboyA0,
  WarboyB0,
  Renegade,
  U250,
} Arch;

typedef enum CoreRangeType {
  All,
  Range,
} CoreRangeType;

typedef enum CoreStatus {
  Available,
  Occupied,
  Unavailable,
} CoreStatus;

typedef enum DeviceMode {
  Single,
  Fusion,
  MultiCore,
} DeviceMode;

typedef enum error_code {
  ok = 0,
  invalid_input,
  null_error,
  unsupported_error,
  unavailable_error,
  device_not_found,
  device_busy,
  io_error,
  permission_denied_error,
  unknown_arch_error,
  incompatible_driver_error,
  hwmon_error,
  performance_counter_error,
  unexpected_value_error,
  parse_error,
  unknown_error,
} error_code;

typedef Device *device_handle;

typedef struct ErrorStatesKeyValuePair {
  const char *key;
  uint32_t value;
} ErrorStatesKeyValuePair;

typedef struct ClockFrequency {
  const char *name;
  const char *unit;
  uint32_t value;
} ClockFrequency;

typedef struct CoreRange {
  enum CoreRangeType range_type;
  uint8_t start;
  uint8_t end;
} CoreRange;

typedef struct DeviceFile {
  uint8_t device_index;
  struct CoreRange core_range;
  const char *path;
  enum DeviceMode mode;
} DeviceFile;

typedef PerformanceCounter *performance_counter_handle;

typedef struct PerformanceCounterPair {
  struct DeviceFile DeviceFile;
  performance_counter_handle PerformanceCounterHandle;
} PerformanceCounterPair;

typedef struct CoreStatusPair {
  uint8_t core_index;
  enum CoreStatus status;
} CoreStatusPair;

enum error_code list_devices(device_handle **output, uint8_t *output_len);

void destroy_device_handles(device_handle *raw, uint8_t len);

enum error_code get_device(uint8_t idx, device_handle *output);

void destroy_device_handle(device_handle device);

enum error_code get_device_name(device_handle handle, char **output);

enum error_code get_device_index(device_handle handle, uint8_t *output);

enum error_code get_device_arch(device_handle handle, enum Arch *output);

enum error_code get_device_aliveness(device_handle handle, bool *output);

enum error_code get_device_error_states(device_handle handle,
                                        struct ErrorStatesKeyValuePair **output,
                                        uintptr_t *output_len);

void destroy_error_states(struct ErrorStatesKeyValuePair *raw, uint8_t len);

enum error_code get_device_pci_bus_number(device_handle handle, char **output);

enum error_code get_device_pci_dev_id(device_handle handle, char **output);

enum error_code get_device_serial_number(device_handle handle, char **output);

enum error_code get_device_uuid(device_handle handle, char **output);

enum error_code get_device_firmware_version(device_handle handle, char **output);

enum error_code get_device_driver_version(device_handle handle, char **output);

enum error_code get_device_heartbeat(device_handle handle, uint32_t *output);

enum error_code get_device_clock_frequency(device_handle handle,
                                           struct ClockFrequency **output,
                                           uint8_t *output_len);

void destroy_clock_frequency(struct ClockFrequency *raw, uint8_t len);

enum error_code get_device_numanode(device_handle handle, uint8_t *output);

enum error_code get_device_core_num(device_handle handle, uint8_t *output);

enum error_code get_device_cores(device_handle handle, uint8_t **output, uint8_t *output_len);

enum error_code get_device_files(device_handle handle,
                                 struct DeviceFile **output,
                                 uint8_t *output_len);

void destroy_device_files(struct DeviceFile *raw, uint8_t len);

enum error_code get_device_performance_counters(device_handle handle,
                                                struct PerformanceCounterPair **output,
                                                uint8_t *output_len);

void destroy_performance_counters(struct PerformanceCounterPair *raw, uint8_t len);

enum error_code get_device_core_status(device_handle handle,
                                       uint8_t core_idx,
                                       enum CoreStatus *output);

enum error_code get_device_core_occupied_fd(device_handle handle, uint8_t core_idx, char **output);

enum error_code get_device_all_core_status(device_handle handle,
                                           struct CoreStatusPair **output,
                                           uint8_t *output_len);

void destroy_core_status_pair(struct CoreStatusPair *raw, uint8_t len);
