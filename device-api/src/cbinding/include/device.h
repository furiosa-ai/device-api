#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum furiArch_t {
  warboy = 0,
  rngd,
  rngd_max,
  rngd_s,
} furiArch_t;

typedef enum furiReturn_t {
  ok = 0,
  unknown_error,
} furiReturn_t;

typedef struct furiSystemPhysicalDevice_t furiSystemPhysicalDevice_t;

typedef struct furiDriverVersion_t {
  enum furiArch_t arch;
  uint32_t major;
  uint32_t minor;
  uint32_t patch;
  char metadata[FURI_MAX_HW_METADATA_SIZE];
} furiDriverVersion_t;

typedef struct furiSystemDriverInfo_t {
  uint8_t count;
  struct furiDriverVersion_t driverInfo[FURI_MAX_DRIVER_INFO_SIZE];
} furiSystemDriverInfo_t;

typedef struct furiSystemPhysicalDevice_t furiSystemPhysicalDeviceHandle_t;

typedef struct furiSystemPhysicalDeviceHandles_t {
  uint8_t count;
  furiSystemPhysicalDeviceHandle_t device_handles[FURI_MAX_DRIVER_INFO_DEVICE_SIZE];
} furiSystemPhysicalDeviceHandles_t;

typedef struct furiDeviceInfoDevice_t {
  char bdf[FURI_MAX_DEVICE_BDF_SIZE];
} furiDeviceInfoDevice_t;

typedef struct furiDeviceInfoDeviceHandles_t {
  uint8_t count;
  struct furiDeviceInfoDevice_t device_handles[FURI_MAX_DEVICE_HANDLE_SIZE];
} furiDeviceInfoDeviceHandles_t;

typedef struct furiDeviceInfo_t {
  enum furiArch_t arch;
  char name[FURI_MAX_BUFFER_SIZE];
  char serial[FURI_MAX_BUFFER_SIZE];
  char uuid[FURI_MAX_BUFFER_SIZE];
  uint32_t core_num;
} furiDeviceInfo_t;

typedef struct furiDeviceFirmwareVersion_t {
  enum furiArch_t arch;
  uint32_t major;
  uint32_t minor;
  uint32_t patch;
  char metadata[FURI_MAX_HW_METADATA_SIZE];
} furiDeviceFirmwareVersion_t;

typedef struct furiDeviceInfoDeviceHwInfo_t {
  char bdf[FURI_MAX_BUFFER_SIZE];
  char pci_dev_id[FURI_MAX_BUFFER_SIZE];
  struct furiDeviceFirmwareVersion_t firmware_version;
  struct furiDriverVersion_t driver_version;
  uint32_t numa_node;
} furiDeviceInfoDeviceHwInfo_t;

enum furiReturn_t furiInit(void);

enum furiReturn_t furiShutdown(void);

enum furiReturn_t furiSystemGetDriverInfo(struct furiSystemDriverInfo_t *systemDriverInfo);

enum furiReturn_t furiSystemGetSrIovCapability(bool *supported);

enum furiReturn_t furiSystemGetPhysicalDeviceInfo(struct furiSystemPhysicalDeviceHandles_t *devices);

enum furiReturn_t furiSystemGetPhysicalDeviceSrIovCapability(furiSystemPhysicalDeviceHandle_t handle,
                                                             bool *supported);

enum furiReturn_t furiSystemGetPhysicalDeviceMaxVfNum(furiSystemPhysicalDeviceHandle_t handle,
                                                      uint8_t *num);

enum furiReturn_t furiSystemGetPhysicalDeviceVfConfig(furiSystemPhysicalDeviceHandle_t handle,
                                                      uint8_t *vfNum);

enum furiReturn_t furiSystemConfigurePhysicalDeviceVf(furiSystemPhysicalDeviceHandle_t handle,
                                                      uint8_t num);

enum furiReturn_t furiSystemUnconfigurePhysicalDeviceVf(furiSystemPhysicalDeviceHandle_t handle);

enum furiReturn_t furiDeviceInfoGetDeviceHandle(struct furiDeviceInfoDeviceHandles_t *info);

enum furiReturn_t furiDeviceInfoGetDeviceHandleByUUID(const char *uuid,
                                                      struct furiDeviceInfoDevice_t *handle);

enum furiReturn_t furiDeviceInfoGetDeviceInfo(struct furiDeviceInfoDevice_t handle,
                                              struct furiDeviceInfo_t *info);

enum furiReturn_t furiDeviceInfoGetDeviceHwInfo(struct furiDeviceInfoDevice_t handle,
                                                struct furiDeviceInfoDeviceHwInfo_t *info);
