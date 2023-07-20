import glob
import os

from furiosa_native_device import Arch, DeviceConfig, DeviceMode
from furiosa_native_device.sync import (
    find_device_files,
    get_device,
    get_device_file,
    list_devices,
)


def get_first_device_name(pattern):
    return sorted(glob.glob(pattern))[0].split("/")[-1]


def test_list_devices():
    dev_name = get_first_device_name("/dev/npu*")
    devices = list_devices()
    assert devices[0].name() == dev_name


def test_get_device():
    dev_name = get_first_device_name("/dev/npu*")
    dev_idx = int(dev_name.replace("npu", ""))
    device = get_device(dev_idx)
    assert device.name() == dev_name


def test_find_device_files():
    dev_name = get_first_device_name("/dev/npu*pe0-1")
    config = DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=1)
    devices = find_device_files(config)
    assert devices[0].filename() == dev_name


def test_find_device_files_err():
    dev_name = get_first_device_name("/dev/npu*pe0")
    config = DeviceConfig.from_str(dev_name)
    fd = os.open(f"/dev/{dev_name}", os.O_RDWR)
    try:
        _ = find_device_files(config)
    except Exception as e:
        assert str(e).endswith("found but still in use")
    finally:
        os.close(fd)


def test_get_device_file():
    dev_name = get_first_device_name("/dev/npu*pe1")
    device_file = get_device_file(dev_name)
    assert device_file.filename() == dev_name
