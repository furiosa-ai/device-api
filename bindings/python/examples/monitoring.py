import asyncio
import os
import time

from furiosa_native_device import list_devices


def print_sensor_value(category, sensor_values):
    print(f"======= {category} =======")
    for sensor_value in sensor_values:
        print(f"{sensor_value.label}: {sensor_value.value}")
    print()


async def main():
    devices = await list_devices()

    while True:
        for device in devices:
            name = device.name()
            fetcher = device.get_hwmon_fetcher()

            currents = await fetcher.read_currents()
            voltages = await fetcher.read_voltages()
            powers = await fetcher.read_powers_average()
            temperatures = await fetcher.read_temperatures()

            os.system("clear")
            print(f"NPU: {name}")
            print_sensor_value("CURRENTS", currents)
            print_sensor_value("VOLTAGES", voltages)
            print_sensor_value("POWERS", powers)
            print_sensor_value("TEMPERATURES", temperatures)

        time.sleep(1)


if __name__ == "__main__":
    asyncio.run(main())
