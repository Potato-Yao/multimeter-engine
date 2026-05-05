#[cfg(feature = "stress-test")]
use crate::external_program::stress_test_manager::TestKind;
use anyhow::{Result, anyhow};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Program {
    start_command: Command,
    pre_args: Option<Vec<String>>,
    args_set: Option<Vec<Vec<String>>>,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    standalone: bool, // whether to kill the process when the instance is dropping
}

impl Program {
    fn new<T, A, I, S, II, SS>(start_command: T, pre_args: Option<II>, args_set: Option<A>) -> Self
    where
        T: AsRef<OsStr>,
        A: IntoIterator<Item = I>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
        II: IntoIterator<Item = SS>,
        SS: Into<String>,
    {
        Self {
            start_command: Command::new(start_command),
            pre_args: pre_args.map(|p| p.into_iter().map(|e| e.into()).collect()),
            args_set: args_set.map(|a| {
                a.into_iter()
                    .map(|e| e.into_iter().map(|e| e.into()).collect())
                    .collect()
            }),
            process: None,
            stdin: None,
            stdout: None,
            standalone: false,
        }
    }

    /// command is which once been called will execute automatically and finish itself after everything has done.
    /// command will run on bash on linux, cmd on windows
    pub fn new_command<T, A, I, S>(start_command: T, args_set: Option<A>) -> Self
    where
        T: AsRef<OsStr>,
        A: IntoIterator<Item = I>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let pre_args = if cfg!(windows) {
            Some(vec!["cmd", "/C"])
        } else {
            None
        };

        Self::new(start_command, pre_args, args_set)
    }

    /// this runs the tool under directory `externals`
    /// for example, if you want to run `stress` at `./externals/linux/stress`, the `tool_path` should be `stress`.
    /// the tool directory will be found automatically
    pub fn new_external_tool<T, A, I, S>(tool_path: T, args_set: Option<A>) -> Self
    where
        T: AsRef<str>,
        A: IntoIterator<Item = I>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(
            get_local_path(tool_path.as_ref()),
            None::<I>,
            args_set,
        )
    }

    /// after calling it, drop the instance of Program will not kill the process it calls
    pub fn make_process_standalone(&mut self) {
        self.standalone = true;
    }

    pub fn start(&mut self, args_index: Option<usize>) -> Result<()> {
        if self.is_running() {
            return Err(anyhow!("Program is already running"));
        }
        if args_index.is_some() && self.args_set.is_none() {
            return Err(anyhow!("This program has no preset arguments"));
        }

        if let Some(pre_args) = &self.pre_args {
            self.start_command.args(pre_args);
        }

        if let Some(index) = args_index {
            if index >= self.args_set.as_ref().unwrap().len() {
                return Err(anyhow!("Index out of bounds of arguments set"));
            }

            let args = &self.args_set.as_ref().unwrap()[args_index.unwrap()];
            self.start_command.args(args);
        }

        self.start_command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut process = self.start_command.spawn()?;
        self.stdin = Some(
            process
                .stdin
                .take()
                .ok_or(anyhow!("Failed to open stdin"))?,
        );
        self.stdout = Some(
            process
                .stdout
                .take()
                .ok_or(anyhow!("Failed to open stdout"))?,
        );
        self.process = Some(process);

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        // it makes sure the unwrap below is safe
        if !self.is_running() {
            return Err(anyhow!("Program is not running"));
        }

        self.process.as_mut().unwrap().kill()?;
        self.process = None;
        self.stdin = None;
        self.stdout = None;

        Ok(())
    }

    pub fn write<T>(&mut self, content: T) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        if !self.is_running() {
            return Err(anyhow!("Program is not running"));
        }

        let stdin = self.stdin.as_mut().unwrap();
        stdin.write_all(content.as_ref())?;
        stdin.flush()?;

        Ok(())
    }

    pub fn read(&mut self) -> Result<String> {
        if !self.is_running() {
            return Err(anyhow!("Program is not running"));
        }

        let mut output = String::new();
        let mut buffer = [0; 1];
        loop {
            match self.stdout.as_mut().unwrap().read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    let ch = buffer[0] as char;
                    output.push(ch);
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(output)
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

#[cfg(feature = "stress-test")]
impl Program {
    pub fn get_test(kind: TestKind) -> Self {
        #[cfg(target_os = "linux")]
        match kind {
            TestKind::Cpu => {
                Self::new_external_tool("stress", Some(vec![vec!["--quiet", "--cpu", "16"]]))
            }
            TestKind::Gpu => Self::new_external_tool("gpu_burn", Some(vec![vec!["-h"]])),
            TestKind::Ram => {
                Self::new_external_tool("stress", Some(vec![vec!["--quiet", "--vm", "16"]]))
            }
        }
        // todo windows side
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        if self.is_running() && !self.standalone {
            self.close().unwrap();
        }
    }
}

pub fn get_local_path(tool_path: &str) -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    #[cfg(test)]
    path.pop();

    path.push("externals");
    path.push(tool_path);

    path.to_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        let mut p = Program::new_command("head", Some(vec![vec!["-c", "2"]]));

        p.start(Some(0)).unwrap();
        p.write("hi").unwrap();
        assert_eq!(p.read().unwrap(), "hi");
    }

    #[test]
    fn test_external_tool() {
        let mut p = Program::new_external_tool("ui-sample", None::<Vec<Vec<String>>>);

        p.start(None).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(10));
        p.close().unwrap();
    }

    #[test]
    #[ignore]
    fn test_drop() {
        {
            let mut p = Program::new_external_tool("ui-sample", None::<Vec<Vec<String>>>);

            p.start(None).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    #[test]
    #[ignore]
    fn test_standalone() {
        let mut p = Program::new_external_tool("ui-sample", None::<Vec<Vec<String>>>);
        p.make_process_standalone();

        p.start(None).unwrap();
    }

    #[test]
    fn test_stress_test() {
        #[cfg(feature = "stress-test")]
        {
            let mut test = Program::get_test(TestKind::Cpu);
            test.start(Some(0)).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(10));
            test.close().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }
}
