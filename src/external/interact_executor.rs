use std::io::{Read, Write};

pub const EOF: &str = "\0";

pub struct InteractExecutor {
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
}

impl InteractExecutor {
    pub fn build(command: &str) -> Result<Self, String> {
        let command = shell_words::split(command).map_err(|e| e.to_string())?;
        let mut process = std::process::Command::new(&command[0])
            .args(&command[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        let stdin = process.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = process.stdout.take().ok_or("Failed to open stdout")?;

        Ok(InteractExecutor { stdin, stdout })
    }

    pub fn execute(&mut self, input: Option<&str>) -> Result<String, String> {
        self.execute_until(input, EOF)
    }

    pub fn execute_until(&mut self, input: Option<&str>, wait_for: &str) -> Result<String, String> {
        let content_str = match input {
            Some(content) => format!("{}\n", content),
            None => "".to_string(),
        };
        self.stdin.write_all(content_str.as_bytes()).map_err(|e| e.to_string())?;
        self.stdin.flush().map_err(|e| e.to_string())?;

        let mut output = String::new();
        let mut buffer = [0; 1];
        loop {
            match self.stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    let ch = buffer[0] as char;
                    output.push(ch);
                    if output.ends_with(wait_for) {
                        break;
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
        }

        Ok(output)
    }

    pub fn consume_until(&mut self, wait_for: &str) -> Result<String, String> {
        self.execute_until(None, wait_for)
    }
}
