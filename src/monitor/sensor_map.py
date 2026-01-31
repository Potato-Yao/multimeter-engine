import sys
from pathlib import Path

missing_item = "__MISSING__"

sensors = [
    (missing_item, missing_item, missing_item, "cpu_name", "string"),
    ("CPU Package", "equals", "Temperature", "cpu_temperature", "double"),
    ("CPU Core #1", "equals", "Temperature", "cpu_temperature_first", "double"),
    (r"^CPU Core #\d{1,2}$", "match", "Temperature", "cpu_temperature_last", "double"),
    ("CPU Core #1 Distance to TjMax", "equals", "Temperature", "cpu_tjmax_first", "double"),
    (r"^CPU Core #\d{1,2} Distance to TjMax", "match", "Temperature", "cpu_tjmax_last", "double"),
    ("CPU Package", "equals", "Power", "cpu_power", "double"),
    ("CPU Core #1", "equals", "Voltage", "cpu_voltage_first", "double"),
    (r"^CPU Core #\d{1,2}$", "match", "Voltage", "cpu_voltage_last", "double"),
    ("CPU Core", "equals", "Voltage", "cpu_voltage", "double"),
    ("CPU Core #1", "equals", "Clock", "cpu_clock_first", "double"),
    (r"^CPU Core #\d{1,2}$", "match", "Clock", "cpu_clock_last", "double"),
    (missing_item, missing_item, missing_item, "cpu_clock_avg", "double"),
    (missing_item, missing_item, missing_item, "cpu_clock_rms", "double"),
    (missing_item, missing_item, missing_item, "cpu_clock_max", "double"),
    ("CPU Total", "equals", "Load", "cpu_usage", "double"),
    ("CPU Core #1", "equals", "Load", "cpu_usage_first", "double"),
    (r"^CPU Core #\d{1,2}$", "match", "Load", "cpu_usage_last", "double"),
    (missing_item, missing_item, missing_item, "gpu_name", "string"),
    ("GPU Core", "equals", "Temperature", "gpu_temperature", "double"),
    ("GPU Package", "equals", "Power", "gpu_power", "double"),
    (missing_item, missing_item, missing_item, "gpu_voltage", "double"),
    ("GPU Core", "equals", "Clock", "gpu_clock_rms", "double"),
    ("GPU Memory", "equals", "Clock", "gpu_mem_clock_rms", "double"),
    ("GPU Core", "equals", "Load", "gpu_usage", "double"),
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
    ("Temperature 1", "equals", "Temperature", "disk_temperature_first", "double"),
    (r"^Temperature \d{1,2}$", "match", "Temperature", "disk_temperature_last", "double"),
    (missing_item, missing_item, missing_item, "disk_partition", "string array"),
    (missing_item, missing_item, missing_item, "disk_disk", "string array"),
    (missing_item, missing_item, missing_item, "disk_partition_detail", "string array"),
    (missing_item, missing_item, missing_item, "disk_disk_detail", "string array"),
]


def warning_message(pos: str) -> str:
    return f"// THE CODE {pos} IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD"


def get_valid_sensors():
    return [s for s in sensors if s[0] != missing_item]


def gen_windows_block() -> str:
    below_warning = warning_message("BELOW")
    above_warning = warning_message("ABOVE")

    lines: list[str] = [below_warning]

    matches = [s for s in  sensors if s[1] == "match"]
    for sensor in matches:
        lines.append(f"let regex_{sensor[3]} = regex::Regex::new(r\"{sensor[0]}\").unwrap();")

    lines.append(r"for sensor in sensors {")

    valid = get_valid_sensors()

    for i, sensor in enumerate(valid):
        name_check, match_type, info_check, query_name, _ty = sensor

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

    valid = get_valid_sensors()
    names = [s[3] for s in valid]

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
