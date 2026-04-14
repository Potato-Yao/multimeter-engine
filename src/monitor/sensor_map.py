import sys
from pathlib import Path

missing_item = "__MISSING__"

# (query_name, match_name, match_type, info_check, type)
sensors = [
    ("cpu_name", missing_item, missing_item, missing_item, "string"),
    ("cpu_temperature", "CPU Package", "equals", "Temperature", "double"),
    ("cpu_temperature", "Core (Tctl/Tdie)", "equals", "Temperature", "double"),
    ("cpu_temperature_first", "CPU Core #1", "equals", "Temperature", "double"),
    ("cpu_temperature_last", r"^CPU Core #\d{1,2}$", "match", "Temperature", "double"),
    ("cpu_tjmax_first", "CPU Core #1 Distance to TjMax", "equals", "Temperature", "double"),
    ("cpu_tjmax_last", r"^CPU Core #\d{1,2} Distance to TjMax", "match", "Temperature", "double"),
    ("cpu_power", "CPU Package", "equals", "Power", "double"),
    ("cpu_power", "Package", "equals", "Power", "double"),
    ("cpu_voltage_first", "CPU Core #1", "equals", "Voltage", "double"),
    ("cpu_voltage_last", r"^CPU Core #\d{1,2}$", "match", "Voltage", "double"),
    ("cpu_voltage", "CPU Core", "equals", "Voltage", "double"),
    ("cpu_voltage", "Core (SVI2 TFN)", "equals", "Voltage", "double"),
    ("cpu_clock_first", "CPU Core #1", "equals", "Clock", "double"),
    ("cpu_clock_first", "Core #1", "equals", "Clock", "double"),
    ("cpu_clock_last", r"^CPU Core #\d{1,2}$", "match", "Clock", "double"),
    ("cpu_clock_last", r"^Core #\d{1,2}$", "match", "Clock", "double"),
    ("cpu_clock_avg", missing_item, missing_item, missing_item, "double"),
    ("cpu_clock_rms", missing_item, missing_item, missing_item, "double"),
    ("cpu_clock_max", missing_item, missing_item, missing_item, "double"),
    ("cpu_usage", "CPU Total", "equals", "Load", "double"),
    ("cpu_usage_first", "CPU Core #1", "equals", "Load", "double"),
    ("cpu_usage_last", r"^CPU Core #\d{1,2}$", "match", "Load", "double"),
    ("gpu_name", missing_item, missing_item, missing_item, "string"),
    ("gpu_temperature", "GPU Core", "equals", "Temperature", "double"),
    ("gpu_temperature", "GPU VR SoC", "equals", "Temperature", "double"),
    ("gpu_power", "GPU Package", "equals", "Power", "double"),
    ("gpu_power", "GPU Core", "equals", "Power", "double"),
    ("gpu_clock_rms", "GPU Core", "equals", "Clock", "double"),
    ("gpu_mem_clock_rms", "GPU Memory", "equals", "Clock", "double"),
    ("gpu_usage", "GPU Core", "equals", "Load", "double"),
    ("mem_percentage", "Memory", "equals", "Load", "double"),
    ("mem_available", "Memory Available", "equals", "Data", "double"),
    ("mem_used", "Memory Used", "equals", "Data", "double"),
    ("bat_capacity_max", "Fully-Charged Capacity", "equals", "Energy", "double"),
    ("bat_capacity_remain", "Remaining Capacity", "equals", "Energy", "double"),
    ("bat_capacity_designed", "Designed Capacity", "equals", "Energy", "double"),
    ("bat_voltage", "Voltage", "equals", "Voltage", "double"),
    ("bat_rate", "Charge Rate", "equals", "Power", "double"),
    ("bat_rate", "Discharge Rate", "equals", "Power", "double"),
    ("bat_rate", "Charge/Discharge Rate", "equals", "Power", "double"),
    ("bat_state", missing_item, missing_item, missing_item, "boolean"),
    ("bat_count", missing_item, missing_item, missing_item, "int"),
    ("os_activated", missing_item, missing_item, missing_item, "boolean"),
    ("disk_temperature_first", "Temperature 1", "equals", "Temperature", "double"),
    ("disk_temperature_last", r"^Temperature \d{1,2}$", "match", "Temperature", "double"),
    ("disk_partition", missing_item, missing_item, missing_item, "string array"),
    ("disk_disk_size", missing_item, missing_item, missing_item, "string array"),
    ("os_name", missing_item, missing_item, missing_item, "string"),
    ("os_kernel_version", missing_item, missing_item, missing_item, "string"),
    ("os_version", missing_item, missing_item, missing_item, "string"),
    ("os_host_name", missing_item, missing_item, missing_item, "string"),
]


def warning_message(pos: str) -> str:
    return f"// THE CODE {pos} IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD"


def get_valid_lhm_sensors():
    return [s for s in sensors if s[1] != missing_item]


def gen_windows_block() -> str:
    below_warning = warning_message("BELOW")
    above_warning = warning_message("ABOVE")

    lines: list[str] = [below_warning]

    matches = [s for s in sensors if s[2] == "match"]
    for sensor in matches:
        query_name, match_name, _match_type, _info_check, _ty = sensor
        lines.append(f"let regex_{query_name} = regex::Regex::new(r\"{match_name}\").unwrap();")

    lines.append(r"for sensor in sensors {")

    valid = get_valid_lhm_sensors()

    for i, sensor in enumerate(valid):
        query_name, name_check, match_type, info_check, _ty = sensor

        if match_type == "equals":
            name_condition = f'sensor.name == "{name_check}"'
        elif match_type == "contains":
            name_condition = f'sensor.name.contains("{name_check}")'
        elif match_type == "match":
            name_condition = f'regex_{query_name}.is_match(&sensor.name)'

        condition = f'{name_condition} && sensor.info == "{info_check}"'

        if i == 0:
            lines.append(f"    if {condition} {{")
        else:
            lines.append(f"    }} else if {condition} {{")

        lines.append(f'        map.insert("{query_name}".to_string(), sensor.index);')

    lines.append("    }")
    lines.append("}")
    lines.append(above_warning)

    return "\n".join(lines) + "\n"


def gen_mod_block() -> str:
    below_warning = warning_message("BELOW")
    above_warning = warning_message("ABOVE")

    names = [s[0] for s in sensors]

    unique_sorted = sorted(set(names))

    lines: list[str] = [below_warning]
    lines.append("pub static ref QUERY_STATEMENTS: Vec<&'static str> = vec![")
    for n in unique_sorted:
        lines.append(f'    "{n}",')
    lines.append("];")
    lines.append(above_warning)

    return "\n".join(lines) + "\n"


def replace_between_markers_linewise(file_path: Path, new_block: str):
    lines = file_path.read_text(encoding="utf-8").splitlines(True)

    def is_below(line: str) -> bool:
        return "// THE CODE BELOW IS SCRIPT GENERATED" in line

    def is_above(line: str) -> bool:
        return "// THE CODE ABOVE IS SCRIPT GENERATED" in line

    start = None
    for i, line in enumerate(lines):
        if is_below(line):
            start = i
            break

    if start is None:
        raise RuntimeError(f"Begin marker not found in {file_path}")

    end = None
    for j in range(start + 1, len(lines)):
        if is_above(lines[j]):
            end = j
            break
        if is_below(lines[j]):
            end = j
            break

    if end is None:
        raise RuntimeError(f"End marker not found in {file_path}")

    indent = lines[start][: len(lines[start]) - len(lines[start].lstrip(" \t"))]
    new_lines = [(indent + l if l.strip() else l) for l in new_block.splitlines(True)]

    updated = lines[:start] + new_lines + lines[end + 1 :]
    new_text = "".join(updated)

    file_path.write_text(new_text, encoding="utf-8")


def main(argv: list[str]) -> int:
    script_dir = Path(__file__).resolve().parent

    windows_rs = script_dir / "windows.rs"
    mod_rs = script_dir / "mod.rs"

    if len(argv) >= 2 and argv[1] == "--print":
        print(gen_windows_block())
        print(gen_mod_block())
        return 0

    replace_between_markers_linewise(windows_rs, gen_windows_block())
    replace_between_markers_linewise(mod_rs, gen_mod_block())

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
