import glob

from furiosa_device import Arch, DeviceConfig, DeviceMode
from furiosa_device.sync import find_devices_sync, get_device_sync, list_devices_sync


def get_first_device_name():
    return sorted(glob.glob("/dev/npu*"))[0].split("/")[-1]


def test_list_devices():
    dev_name = get_first_device_name()
    devices = list_devices_sync()
    assert devices[0].name() == dev_name


def test_find_devices():
    dev_name = get_first_device_name()
    config = DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=1)
    devices = find_devices_sync(config)
    assert devices[0].filename() == f"{dev_name}pe0-1"


def test_get_device():
    device = get_device_sync("npu0pe1")
    assert device.filename() == "npu0pe1"
