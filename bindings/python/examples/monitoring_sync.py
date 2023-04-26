import os
import time

from furiosa_device.sync import list_devices_sync


def print_sensor_value(category, sensor_values):
    print(f"======= {category} =======")
    for sensor_value in sensor_values:
        print(f"{sensor_value.label}: {sensor_value.value}")
    print()


def main():
    devices = list_devices_sync()

    while True:
        for device in devices:
            name = device.name()
            fetcher = device.get_hwmon_fetcher()

            currents = fetcher.read_currents()
            voltages = fetcher.read_voltages()
            powers = fetcher.read_powers_average()
            temperatures = fetcher.read_temperatures()

            os.system("clear")
            print(f"NPU: {name}")
            print_sensor_value("CURRENTS", currents)
            print_sensor_value("VOLTAGES", voltages)
            print_sensor_value("POWERS", powers)
            print_sensor_value("TEMPERATURES", temperatures)

        time.sleep(1)


if __name__ == "__main__":
    main()
