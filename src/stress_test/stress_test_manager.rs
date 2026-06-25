use std::cmp::PartialEq;
use crate::external_program::program::Program;
use crate::monitor;
use anyhow::{Result, anyhow};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// three bits, for cpu, gpu and ram respectively
pub type TestCombine = u8;
pub const CPU_TEST: TestCombine = 0b100;
pub const GPU_TEST: TestCombine = 0b010;
pub const RAM_TEST: TestCombine = 0b001;
pub const CPU_GPU_TEST: TestCombine = CPU_TEST | GPU_TEST;
pub const ALL_TEST: TestCombine = CPU_TEST | GPU_TEST | RAM_TEST;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestState {
    Waiting,
    Well,
    OverHeat,
    LowPower,
    NotStable,
    Unknown,
    NotWorking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestMode {
    Infinity,
    AutoJudge,
    TimeLimit(u64),
}

pub struct StressTestManager {
    cpu_test: Option<Program>,
    gpu_test: Option<Program>,
    ram_test: Option<Program>,
    state: Arc<Mutex<[TestState; 4]>>, // [manager state, cpu state, gpu state, ram state]
    combine: TestCombine,
    mode: Option<TestMode>,
    state_worker_stop: Option<Arc<AtomicBool>>,
    state_worker_handle: Option<JoinHandle<()>>,
}

pub enum TestKind {
    Cpu,
    Gpu,
    Ram,
}

#[cfg(feature = "stress-test")]
impl Program {
    pub fn get_test(kind: TestKind) -> Option<Self> {
        if cfg!(target_os = "linux") {
            match kind {
                TestKind::Cpu => {
                    Some(Self::new_external_tool("stress").args(vec!["--quiet", "--cpu", "16"]))
                }
                TestKind::Gpu => {
                    Some(Self::new_external_tool("gpu_burn").preserve_working_dir())
                }
                TestKind::Ram => {
                    Some(Self::new_external_tool("stress").args(vec!["--quiet", "--vm", "16"]))
                }
            }
        } else {
            match kind {
                _ => None,
            }
        }
        // todo windows side
    }
}

impl StressTestManager {
    pub fn new() -> Self {
        Self {
            cpu_test: Program::get_test(TestKind::Cpu).into(),
            gpu_test: Program::get_test(TestKind::Gpu).into(),
            ram_test: Program::get_test(TestKind::Ram).into(),
            state: Arc::new(Mutex::new([
                TestState::NotWorking,
                TestState::NotWorking,
                TestState::NotWorking,
                TestState::NotWorking,
            ])),
            combine: 0,
            mode: None,
            state_worker_stop: None,
            state_worker_handle: None,
        }
    }

    pub fn start_test(&mut self, combine: TestCombine, mode: TestMode) -> Result<()> {
        if self.mode.is_some() {
            return Err(anyhow!("test already started"));
        }

        monitor::init()?;



        self.combine = combine;
        self.update_state([TestState::Waiting, TestState::Well, TestState::Well, TestState::Well]);
        self.mode = Some(mode);

        self.start_state_worker()?;

        if mode == TestMode::AutoJudge {
        } else {
            for (mask, kind) in [
                (CPU_TEST, TestKind::Cpu),
                (GPU_TEST, TestKind::Gpu),
                (RAM_TEST, TestKind::Ram),
            ] {
                if combine & mask != 0 {
                    self.start(kind)?;
                }
            }
            let mut next_state = self.read_state();
            next_state[0] = TestState::Well;
            self.update_state(next_state);
        }

        Ok(())
    }

    pub fn stop_test(&mut self) -> Result<()> {
        if self.mode.is_none() {
            return Ok(());
        }

        self.stop_state_worker();

        for (mask, kind) in [
            (CPU_TEST, TestKind::Cpu),
            (GPU_TEST, TestKind::Gpu),
            (RAM_TEST, TestKind::Ram),
        ] {
            if self.combine & mask != 0 {
                self.close(kind)?;
            }
        }

        self.combine = 0;
        self.update_state([
            TestState::NotWorking,
            TestState::NotWorking,
            TestState::NotWorking,
            TestState::NotWorking,
        ]);
        self.mode = None;

        Ok(())
    }

    pub fn get_state(&self) -> [TestState; 4] {
        self.read_state()
    }

    fn start(&mut self, kind: TestKind) -> Result<()> {
        let program = self.program_mut(kind)?;
        program
            .start(if program.has_args() { Some(0) } else { None })
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }

    fn close(&mut self, kind: TestKind) -> Result<()> {
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

    fn start_state_worker(&mut self) -> Result<()> {
        if self.state_worker_handle.is_some() {
            return Err(anyhow!("state worker already running"));
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop_flag);
        let thread_state = Arc::clone(&self.state);

        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                if let Ok(mut state) = thread_state.lock() {
                    Self::generate_state(&mut state);
                }

                thread::sleep(Duration::from_secs(1));
            }
        });

        self.state_worker_stop = Some(stop_flag);
        self.state_worker_handle = Some(handle);

        Ok(())
    }

    fn stop_state_worker(&mut self) {
        if let Some(stop_flag) = self.state_worker_stop.take() {
            stop_flag.store(true, Ordering::Relaxed);
        }

        if let Some(handle) = self.state_worker_handle.take() {
            let _ = handle.join();
        }
    }

    fn read_state(&self) -> [TestState; 4] {
        match self.state.lock() {
            Ok(state) => *state,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }

    fn update_state(&self, next_state: [TestState; 4]) {
        match self.state.lock() {
            Ok(mut state) => *state = next_state,
            Err(poisoned) => *poisoned.into_inner() = next_state,
        }
    }

    fn generate_state(state: &mut [TestState; 4]) {}
}

impl Drop for StressTestManager {
    fn drop(&mut self) {
        self.stop_state_worker();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn test_cpu_stress() {
        let mut manager = super::StressTestManager::new();
        manager.start(super::TestKind::Cpu).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        manager.close(super::TestKind::Cpu).unwrap();
    }

    #[test]
    #[ignore]
    fn test_gpu_stress() {
        let mut manager = super::StressTestManager::new();
        manager.start(super::TestKind::Gpu).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        manager.close(super::TestKind::Gpu).unwrap();
    }
}
