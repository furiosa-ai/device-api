from enum import Enum
from typing import Dict, List, Tuple, Union

class Arch(Enum):
    WarboyB0 = ...
    RNGD = ...

class DeviceMode(Enum):
    Single = ...
    Fusion = ...
    MultiCore = ...

class CoreStatusType(Enum):
    Available = ...
    Occupied = ...
    Unavailable = ...

class CoreStatus:
    status_type: CoreStatusType
    value: Union[None, str]
    def __repr(self) -> str: ...

class CoreRangeType(Enum):
    All = ...
    Range = ...

class CoreRange:
    range_type: CoreRangeType
    value: Union[None, Tuple[int, int]]

    def contains(self, int) -> bool: ...
    def __repr(self) -> str: ...

class SensorValue:
    label: str
    value: int
    def __repr(self) -> str: ...

class ClockFrequency:
    name: str
    unit: str
    value: int
    def __repr(self) -> str: ...

class DeviceConfig:
    def __new__(
        cls,
        _arch: Arch = Arch.WarboyB0,
        mode: DeviceMode = DeviceMode.Fusion,
        count: int = 1,
    ) -> DeviceConfig: ...
    @classmethod
    def from_env(cls, key: str) -> DeviceConfig: ...
    @classmethod
    def from_str(cls, key: str) -> DeviceConfig: ...
    def __repr(self) -> str: ...

class DeviceFile:
    def path(self) -> str: ...
    def filename(self) -> str: ...
    def devfile_index(self) -> int: ...
    def core_range(self) -> CoreRange: ...
    def mode(self) -> DeviceMode: ...
    def __repr(self) -> str: ...

class Fetcher:
    async def read_currents(self) -> List[SensorValue]: ...
    async def read_voltages(self) -> List[SensorValue]: ...
    async def read_powers_average(self) -> List[SensorValue]: ...
    async def read_temperatures(self) -> List[SensorValue]: ...

class Utilization:
    def npu_utilization(self) -> float: ...
    def computation_ratio(self) -> float: ...
    def io_ratio(self) -> float: ...
    def __repr(self) -> str: ...

class PerformanceCounter:
    def cycle_count(self) -> int: ...
    def task_execution_cycle(self) -> int: ...
    def tensor_execution_cycle(self) -> int: ...
    def calculate_increased(self, PerformanceCounter) -> PerformanceCounter: ...
    def calculate_utilization(self, PerformanceCounter) -> Utilization: ...
    def __repr(self) -> str: ...

class Device:
    def name(self) -> str: ...
    def devfile_index(self) -> int: ...
    def arch(self) -> Arch: ...
    def alive(self) -> bool: ...
    def atr_error(self) -> List[Dict[str, int]]: ...
    def busname(self) -> str: ...
    def pci_dev(self) -> str: ...
    def device_sn(self) -> str: ...
    def device_uuid(self) -> str: ...
    def firmware_version(self) -> str: ...
    def driver_version(self) -> str: ...
    def heartbeat(self) -> int: ...
    def clock_frequency(self) -> List[ClockFrequency]: ...
    def ctrl_device_led(self, led: Tuple[bool, bool, bool]) -> None: ...
    def core_num(self) -> int: ...
    def cores(self) -> List[int]: ...
    def dev_files(self) -> List[DeviceFile]: ...
    def performance_counters(self) -> List[Tuple[DeviceFile, PerformanceCounter]]: ...
    async def get_status_core(self, core: int) -> List[CoreStatus]: ...
    async def get_status_all(self) -> Dict[int, CoreStatus]: ...
    def get_hwmon_fetcher(self) -> Fetcher: ...
    def __repr(self) -> str: ...

async def list_devices() -> List[Device]: ...
async def get_device(idx: int) -> Device: ...
async def find_device_files(config: DeviceConfig) -> List[DeviceFile]: ...
async def get_device_file(device_name: str) -> DeviceFile: ...

# These are included in furiosa_device.sync module
class FetcherSync(Fetcher):
    def read_currents(self) -> List[SensorValue]: ...
    def read_voltages(self) -> List[SensorValue]: ...
    def read_powers_average(self) -> List[SensorValue]: ...
    def read_temperatures(self) -> List[SensorValue]: ...

class DeviceSync(Device):
    def get_status_core(self, core: int) -> List[CoreStatus]: ...
    def get_status_all(self) -> Dict[int, CoreStatus]: ...
    def get_hwmon_fetcher(self) -> FetcherSync: ...

def list_devices_sync() -> List[DeviceSync]: ...
def get_device_sync(idx: int) -> DeviceSync: ...
def find_device_files_sync(config: DeviceConfig) -> List[DeviceFile]: ...
def get_device_file_sync(device_name: str) -> DeviceFile: ...
