import pytest
from furiosa_device import (
    Arch,
    DeviceConfig,
    DeviceMode,
    find_devices,
    get_device,
    list_devices,
)

# These tests are only for local machine with one WarboyB0 machine


@pytest.mark.asyncio
async def test_list_devices():
    devices = await list_devices()
    assert devices[0].name() == "npu0"


@pytest.mark.asyncio
async def test_find_devices():
    config = DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=1)
    devices = await find_devices(config)
    assert devices[0].filename() == "npu0pe0-1"


@pytest.mark.asyncio
async def test_get_device():
    device = await get_device("npu0pe1")
    assert device.filename() == "npu0pe1"
