use crate::external_program::program::{ExternalProgram, ProgramKind};

use anyhow::{Result, anyhow};

pub struct StressTestManager {
    cpu_test: Option<ExternalProgram>,
    gpu_test: Option<ExternalProgram>,
    ram_test: Option<ExternalProgram>,
}

pub enum TestKind {
    Cpu,
    Gpu,
    Ram,
}

#[cfg(all(feature = "stress-test", target_os = "linux"))]
impl StressTestManager {
    pub fn new() -> Self {
        Self {
            cpu_test: Some(ExternalProgram::new_interpreter(
                "linux/stress",
                ProgramKind::Executable,
                vec![vec!["--quiet", "--cpu", "16"]],
            )),
            gpu_test: Some(ExternalProgram::new_interpreter(
                "linux/gpu_burn",
                ProgramKind::Executable,
                vec![vec!["-h"]],
            )),
            ram_test: Some(ExternalProgram::new_interpreter(
                "linux/stress",
                ProgramKind::Executable,
                vec![vec!["--quiet", "--vm", "16"]],
            )),
        }
    }

    pub fn start(&mut self, kind: TestKind) -> Result<()> {
        self.program_mut(kind)?
            .start(0)
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }

    pub fn close(&mut self, kind: TestKind) -> Result<()> {
        self.program_mut(kind)?.close()
    }

    fn program_mut(&mut self, kind: TestKind) -> Result<&mut ExternalProgram> {
        match kind {
            TestKind::Cpu => self
                .cpu_test
                .as_mut()
                .ok_or_else(|| anyhow!("CPU stress test not available")),
            TestKind::Gpu => self
                .gpu_test
                .as_mut()
                .ok_or_else(|| anyhow!("GPU stress test not available")),
            TestKind::Ram => self
                .ram_test
                .as_mut()
                .ok_or_else(|| anyhow!("RAM stress test not available")),
        }
    }
}

#[cfg(all(feature = "stress-test", test))]
mod tests {
    #[test]
    #[ignore = "runs a real CPU stress test"]
    fn test_cpu_stress() {
        let mut manager = super::StressTestManager::new();
        manager.start(super::TestKind::Cpu).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        manager.close(super::TestKind::Cpu).unwrap();
    }

    #[test]
    #[ignore = "runs a real GPU stress test"]
    fn test_gpu_stress() {
        let mut manager = super::StressTestManager::new();
        manager.start(super::TestKind::Gpu).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        manager.close(super::TestKind::Gpu).unwrap();
    }
}
