use std::time::Duration;
use crate::external_program::interact_executor::{InteractExecutor, EOF};

#[derive(PartialEq)]
pub enum ProgramKind {
    Executable,
    Command,
}

pub struct ExternalProgram {
    path: String,
    args_set: Vec<Vec<String>>,
    interactive: bool,
    program_kind: ProgramKind,
    process: Option<InteractExecutor>,
}

impl ExternalProgram {
    pub fn new_transient(
        path: String,
        program_kind: ProgramKind,
        args_set: Vec<Vec<String>>,
    ) -> Self {
        ExternalProgram {
            path,
            args_set,
            interactive: false,
            program_kind,
            process: None,
        }
    }

    pub fn new_interpreter(
        path: String,
        program_kind: ProgramKind,
        args_set: Vec<Vec<String>>,
    ) -> Self {
        ExternalProgram {
            path,
            args_set,
            interactive: true,
            program_kind,
            process: None,
        }
    }

    /// for transient external programs, starting a program is equivalent to execute it, the return value will be the output of the program.
    /// for interpreter external programs, starting a program will launch the interpreter with the given args, the return value has no meaning.
    pub fn start(&mut self, args_index: usize) -> Result<String, String> {
        if args_index >= self.args_set.len() {
            return Err("Invalid args index".to_string());
        }

        let args = &self.args_set[args_index];
        let command_name = match self.program_kind {
            ProgramKind::Executable => get_local_path(&*self.path),
            ProgramKind::Command => self.path.clone(),
        };

        // use InteractExecutor for interpreter type programs, std::process::Command for transient type programs
        if self.interactive {
            let mut command = format!("{} {}", command_name, args.join(" "));
            #[cfg(windows)]
            {
                if self.program_kind == ProgramKind::Command {
                    command = format!("cmd /C {}", command);
                } else if self.program_kind == ProgramKind::Executable {
                    command = format!("{}", command);
                }
            }

            let process = InteractExecutor::build(&*command)?;
            std::thread::sleep(Duration::from_secs(3));

            self.process = Some(process);
            Ok(String::new())
        } else {
            let mut command;
            if cfg![windows] && self.program_kind == ProgramKind::Command {
                command = std::process::Command::new("cmd");
                command.args(&["/C", &self.path]);
                command.args(args);
            } else {
                command = std::process::Command::new(command_name);
                command.args(args);
            }

            let output = command.output().map_err(|e| e.to_string())?;

            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(result)
            } else {
                let error = String::from_utf8_lossy(&output.stderr).to_string();
                Err(error)
            }
        }
    }

    pub fn interact(
        &mut self,
        message: String,
        wait_for: Option<String>,
    ) -> Result<String, String> {
        if let Some(process) = self.process.as_mut() {
            let output = process
                .execute_until(
                    Some(message.as_str()),
                    wait_for.unwrap_or(EOF.to_string()).as_str(),
                )
                .map_err(|e| e.to_string())?;

            Ok(output)
        } else {
            Err("Process is not running".to_string())
        }
    }

    pub fn consume_initial_output(&mut self, wait_for: String) -> Result<String, String> {
        if let Some(process) = self.process.as_mut() {
            let output = process
                .consume_until(wait_for.as_str())
                .map_err(|e| e.to_string())?;

            Ok(output)
        } else {
            Err("Process is not running".to_string())
        }
    }

    pub fn close(&mut self) {
        if let Some(_) = &mut self.process {
            self.process = None;
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
    use crate::external_program::program::{ExternalProgram, ProgramKind};

    #[test]
    fn test_run_transient_command() {
        let mut program = ExternalProgram::new_transient(
            "echo".to_string(),
            ProgramKind::Command,
            vec![vec!["Hello,".to_string(), "World!".to_string()]],
        );

        match program.start(0) {
            Ok(output) => assert_eq!(output.trim(), "Hello, World!"),
            Err(e) => panic!("Failed to run transient program: {}", e),
        }
    }

    #[test]
    fn test_run_transient_tool() {
        #[cfg(windows)]
        {
            let mut program = ExternalProgram::new_transient(
                "CLINIC_OP/CPU/cpuz_x64.exe".to_string(),
                ProgramKind::Executable,
                vec![vec![]],
            );
            match program.start(0) {
                Ok(_) => (),
                Err(e) => panic!("Failed to run program: {}", e),
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut program = ExternalProgram::new_transient(
                "linux_tools/ui-sample".to_string(),
                ProgramKind::Executable,
                vec![vec!["a".to_string()]],
            );
            match program.start(0) {
                Ok(_) => (),
                Err(e) => panic!("Failed to run program: {}", e),
            }
        }
    }

    #[test]
    fn test_run_interpreter_command() {
        #[cfg(windows)]
        {
            let mut program = ExternalProgram::new_interpreter(
                // "python".to_string(),
                "diskpart".to_string(),
                ProgramKind::Command,
                vec![vec![]],
            );

            if let Err(e) = program.start(0) {
                panic!("Failed to start program: {}", e);
            }

            program
                .consume_initial_output("DISKPART>".to_string())
                .unwrap();

            match program.interact("list disk".to_string(), Some("DISKPART>".to_string())) {
                // match program.interact("print(\"hi\")".to_string(), Some(">>>".to_string())) {
                Ok(output) => {
                    println!("The output: {}", output);
                    assert!(!output.is_empty());
                    assert!(output.to_lowercase().contains("disk"));
                }
                Err(e) => panic!("Interaction failed: {}", e),
            }

            program.close();
        };
    }

    #[test]
    fn test_run_interpreter_tool() {
        #[cfg(windows)]
        {
            let mut program = ExternalProgram::new_interpreter(
                "win-activate/MAS_AIO.cmd".to_string(),
                ProgramKind::Executable,
                vec![vec![]],
            );

            if let Err(e) = program.start(0) {
                panic!("Failed to start program: {}", e);
            }

            std::thread::sleep(Duration::from_secs(2));

            // match program.interact("1".to_string(), None) {
            //     Ok(_) => {}
            //     Err(e) => panic!("Interaction failed: {}", e),
            // }
            //
            program.close();
        };
    }
}