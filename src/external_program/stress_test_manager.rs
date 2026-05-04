use crate::external_program::program::Program;
use anyhow::{Result, anyhow};

pub struct StressTestManager {
    cpu_test: Option<Program>,
    gpu_test: Option<Program>,
    ram_test: Option<Program>,
}

pub enum TestKind {
    Cpu,
    Gpu,
    Ram,
}

#[cfg(target_os = "linux")]
impl StressTestManager {
    pub fn new() -> Self {
        Self {
            cpu_test: Program::get_test(TestKind::Cpu).into(),
            gpu_test: Program::get_test(TestKind::Gpu).into(),
            ram_test: Program::get_test(TestKind::Ram).into(),
        }
    }

    pub fn start(&mut self, kind: TestKind) -> Result<()> {
        self.program_mut(kind)?
            .start(Some(0))
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }

    pub fn close(&mut self, kind: TestKind) -> Result<()> {
        self.program_mut(kind)?.close()
    }

    fn program_mut(&mut self, kind: TestKind) -> Result<&mut Program> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_cpu_stress() {
        let mut manager = super::StressTestManager::new();
        manager.start(super::TestKind::Cpu).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        manager.close(super::TestKind::Cpu).unwrap();
    }

    #[test]
    fn test_gpu_stress() {
        let mut manager = super::StressTestManager::new();
        manager.start(super::TestKind::Gpu).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        manager.close(super::TestKind::Gpu).unwrap();
    }
}
