use std::{
    env, fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const WINDOWS_TOOLS: [&str; 5] = [
    "CLINIC_OP",
    "cpuburn",
    "FurMark_win64",
    "LibreHardwareMonitorWrapper",
    "win-active",
];

const LINUX_TOOLS: [&str; 0] = [];

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_root = Path::new(&manifest_dir).join("externals");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut target_root = PathBuf::from(out_dir);
    target_root.pop();
    target_root.pop();
    target_root.pop();

    // config what to copy
    let walker = WalkDir::new(&src_root).into_iter().filter_entry(|e| {
        let file_name = e.file_name().to_str().unwrap_or("");

        if e.file_type().is_dir() {
            #[cfg(target_os = "linux")]
            if WINDOWS_TOOLS.contains(&file_name) {
                return false;
            }
            #[cfg(target_os = "windows")]
            if LINUX_TOOLS.contains(&file_name) {
                return false;
            }
        }
        true
    });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(&src_root).unwrap();
        let target_path = target_root.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path).ok();
        } else {
            fs::copy(path, &target_path).expect("Failed to copy file");
        }
    }
}
