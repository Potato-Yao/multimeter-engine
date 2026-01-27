import sys

missing_item = "__MISSING__"

sensors = [
    (missing_item, missing_item, missing_item, "cpu_name", "string"),
    ("CPU Package", "equals", "Temperature", "cpu_temperature", "double"),
    ("CPU Package", "equals", "Power", "cpu_power", "double"),
    ("CPU Core", "equals", "Voltage", "cpu_voltage", "double"),
    (missing_item, missing_item, missing_item, "cpu_clock_avg", "double"),
    (missing_item, missing_item, missing_item, "cpu_clock_rms", "double"),
    (missing_item, missing_item, missing_item, "cpu_clock_max", "double"),
    (missing_item, missing_item, missing_item, "cpu_usage", "double"),
    (missing_item, missing_item, missing_item, "gpu_name", "string"),
    ("GPU Core", "equals", "Temperature", "gpu_temperature", "double"),
    ("GPU Package", "equals", "Power", "gpu_power", "double"),
    (missing_item, missing_item, missing_item, "gpu_voltage", "double"),
    (missing_item, missing_item, missing_item, "gpu_clock_rms", "double"),
    (missing_item, missing_item, missing_item, "mem_total", "double"),
    ("Memory Available", "equals", "Data", "mem_available", "double"),
    ("Fully-Charged Capacity", "equals", "Energy", "bat_capacity_max", "double"),
    ("Remaining Capacity", "equals", "Energy", "bat_capacity_remain", "double"),
    ("Designed Capacity", "equals", "Energy", "bat_capacity_designed", "double"),
    ("Voltage", "equals", "Voltage", "bat_voltage", "double"),
    ("Charge Rate", "equals", "Power", "bat_rate", "double"),
    ("Discharge Rate", "equals", "Power", "bat_rate", "double"),
    ("Charge/Discharge Rate", "equals", "Power", "bat_rate", "double"),
    (missing_item, missing_item, missing_item, "bat_state", "boolean"),
    (missing_item, missing_item, missing_item, "os_activated", "boolean"),
    (missing_item, missing_item, missing_item, "disk_partition", "string array"),
    (missing_item, missing_item, missing_item, "disk_disk", "string array"),
    (missing_item, missing_item, missing_item, "disk_partition_detail", "string array"),
    (missing_item, missing_item, missing_item, "disk_disk_detail", "string array"),
]

def warning_message(pos):
    return f"// THE CODE {pos} IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT {sys.argv[0]} INSTEAD"



if __name__ == "__main__":
    below_warning = warning_message("BELOW")
    above_warning = warning_message("ABOVE")
    
    print(below_warning)

    valid_sensors = [s for s in sensors if s[0] != missing_item]

    for i, sensor in enumerate(valid_sensors):
        name_check = sensor[0]
        match_type = sensor[1]
        info_check = sensor[2]
        query_name = sensor[3]

        if match_type == "equals":
            name_condition = f'sensor.name == "{name_check}"'
        elif match_type == "contains":
            name_condition = f'sensor.name.contains("{name_check}")'
        else:
            name_condition = f'sensor.name == "{name_check}"'

        condition = f'{name_condition} && sensor.info == "{info_check}"'

        if i == 0:
            print(f"if {condition} {{")
        else:
            print(f"}} else if {condition} {{")

        print(f'    map.insert("{query_name}".to_string(), sensor.index);')

    print("}")
    print(above_warning)
