import glob

from furiosa_device import Arch, DeviceConfig, DeviceMode
from furiosa_device.sync import find_devices, get_device, list_devices


def get_first_device_name(pattern):
    return sorted(glob.glob(pattern))[0].split("/")[-1]


def test_list_devices():
    dev_name = get_first_device_name("/dev/npu*")
    devices = list_devices()
    assert devices[0].name() == dev_name


def test_find_devices():
    dev_name = get_first_device_name("/dev/npu*pe0-1")
    config = DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=1)
    devices = find_devices(config)
    assert devices[0].filename() == dev_name


def test_get_device():
    dev_name = get_first_device_name("/dev/npu*pe1")
    device = get_device(dev_name)
    assert device.filename() == dev_name
