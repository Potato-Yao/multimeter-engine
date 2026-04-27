use crate::external_program::program::{ExternalProgram, ProgramKind};

pub struct StressTestManager {
    cpu_test: Option<ExternalProgram>,
    gpu_test: Option<ExternalProgram>,
    ram_test: Option<ExternalProgram>,
}

#[cfg(all(feature = "stress-test", target_os = "linux"))]
impl StressTestManager {
    pub fn new() -> Self {
        Self {
            cpu_test: Some(ExternalProgram::new_transient(
                "linux_tools/stress",
                ProgramKind::Executable,
                vec![vec!["--quiet", "--cpu", "32"]],
            )),
            gpu_test: None,
            ram_test: None,
        }
    }
}

#[cfg(all(feature = "stress-test", test))]
mod tests {
    #[test]
    fn test_cpu_stress() {
            let mut manager = super::StressTestManager::new();
            if let Some(cpu_test) = &mut manager.cpu_test {
                let _ = cpu_test.start(0);
            }
    }
}
