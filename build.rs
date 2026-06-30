use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let sensor_map = Path::new(&manifest_dir).join("src/monitor/sensor_map.py");
    println!("cargo:rerun-if-changed={}", sensor_map.display());
    let status = Command::new("python")
        .arg(sensor_map)
        .status()
        .expect("Failed to run sensor_map.py");
    if !status.success() {
        panic!("sensor_map.py execution failed");
    }

    let src_root = Path::new(&manifest_dir).join("externals");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut target_root = PathBuf::from(out_dir);
    target_root.pop();
    target_root.pop();
    target_root.pop();
    let target_root = target_root.join("externals");

    let profile = env::var("PROFILE").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_release = profile == "release";
    let is_windows = target_os == "windows";

    let platform_dir = if is_windows {
        src_root.join("windows")
    } else {
        src_root.join("linux")
    };

    let walker = WalkDir::new(&platform_dir).into_iter().filter_entry(|e| {
        if is_release && is_windows && e.file_name() == "LibreHardwareMonitorWrapper" {
            return false;
        }
        true
    });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(&platform_dir).unwrap();
        let target_path = target_root.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path).ok();
        } else {
            fs::copy(path, &target_path).expect("Failed to copy file");
        }
    }

    if is_release && is_windows {
        let lhm_project =
            platform_dir.join("LibreHardwareMonitorWrapper/LibreHardwareMonitorWrapper.csproj");
        let lhm_out = target_root.join("LibreHardwareMonitorWrapper/build");

        println!("cargo:warning=Building LibreHardwareMonitorWrapper...");
        let status = Command::new("dotnet")
            .arg("publish")
            .arg(lhm_project)
            .arg("-c")
            .arg("Release")
            .arg("-r")
            .arg("win-x64")
            .arg("--self-contained")
            .arg("true")
            .arg("-o")
            .arg(lhm_out)
            .status()
            .expect("Failed to run dotnet publish");

        if !status.success() {
            panic!("LibreHardwareMonitorWrapper build failed");
        }
    }
}
