import glob

from furiosa_device import Arch, DeviceConfig, DeviceMode
from furiosa_device.sync import find_devices, get_device, list_devices


def get_first_device_name():
    return sorted(glob.glob("/dev/npu*"))[0].split("/")[-1]


def test_list_devices():
    dev_name = get_first_device_name()
    devices = list_devices()
    assert devices[0].name() == dev_name


def test_find_devices():
    dev_name = get_first_device_name()
    config = DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=1)
    devices = find_devices(config)
    assert devices[0].filename() == f"{dev_name}pe0-1"


def test_get_device():
    dev_name = get_first_device_name()
    device = get_device(f"{dev_name}pe1")
    assert device.filename() == f"{dev_name}pe1"
