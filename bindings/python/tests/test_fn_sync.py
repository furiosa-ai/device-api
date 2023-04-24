from furiosa_device import (
    Arch,
    DeviceConfig,
    DeviceMode,
)
from furiosa_device.sync import (
    find_devices_sync,
    get_device_sync,
    list_devices_sync,
)


def test_list_devices():
    devices = list_devices_sync()
    assert devices[0].name() == "npu0"


def test_find_devices():
    config = DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=1)
    devices = find_devices_sync(config)
    assert devices[0].filename() == "npu0pe0-1"


def test_get_device():
    device = get_device_sync("npu0pe1")
    assert device.filename() == "npu0pe1"
