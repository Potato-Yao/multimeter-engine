use crate::external::get_local_path;
use chrono::Local;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    GetHardware = 0,
    SetAuto = 1,
    SetValue = 2,
    GetValue = 3,
    Shutdown = 4,
    Update = 5,
}

pub struct LhmHelper {
    process_handle: Child,
    stream: TcpStream,
}

impl LhmHelper {
    const IP: &'static str = "127.0.0.1";
    const DEFAULT_PORT: u16 = 49200;
    const CHECK: &'static [u8] = b"fan-control-check";
    const CHECK_RESPONSE: &'static [u8] = b"fan-control-ok";
    const PORT_FIND_RANGE: u16 = 50;

    pub fn connect() -> io::Result<Self> {
        let wrapper_path =
            get_local_path("LibreHardwareMonitorWrapper/build/LibreHardwareMonitorWrapper.exe");
        Self::connect_custom(PathBuf::from(wrapper_path), 10)
    }

    pub fn connect_custom(wrapper_path: PathBuf, timeout_secs: u64) -> io::Result<Self> {
        let mut log_dir = std::env::current_exe()?;
        log_dir.pop();
        log_dir.push("logs");
        fs::create_dir_all(&log_dir)?;

        let ts = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let log_file_path = log_dir.join(format!("LibreHardwareMonitorWrapper_{}.log", ts));
        let log_file = fs::File::create(&log_file_path)?;

        let mut child = ProcessCommand::new(&wrapper_path)
            .arg("--log=error")
            .current_dir(wrapper_path.parent().unwrap_or(&wrapper_path))
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::piped())
            .spawn()?;

        let start_time = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let mut stream: Option<TcpStream> = None;

        while start_time.elapsed() < timeout {
            if let Ok(Some(status)) = child.try_wait() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Wrapper exited early with status: {}", status),
                ));
            }

            for port in Self::DEFAULT_PORT..(Self::DEFAULT_PORT + Self::PORT_FIND_RANGE) {
                let addr = format!("{}:{}", Self::IP, port)
                    .parse::<SocketAddr>()
                    .unwrap();
                if let Ok(s) = TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
                    stream = Some(s);
                    break;
                }
            }

            if stream.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }

        let mut stream = stream.ok_or_else(|| {
            io::Error::new(io::ErrorKind::TimedOut, "Connection to LHM timed out")
        })?;

        stream.write_all(Self::CHECK)?;
        stream.flush()?;

        let mut buffer = vec![0u8; Self::CHECK_RESPONSE.len()];
        stream.read_exact(&mut buffer)?;

        if buffer != Self::CHECK_RESPONSE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid response from LHM",
            ));
        }

        Ok(LhmHelper {
            process_handle: child,
            stream,
        })
    }

    fn send_command(&mut self, cmd: Command) -> io::Result<()> {
        let code = (cmd as i32).to_le_bytes();
        self.stream.write_all(&code)?;
        self.stream.flush()
    }

    fn send_int(&mut self, val: i32) -> io::Result<()> {
        self.stream.write_all(&val.to_le_bytes())?;
        self.stream.flush()
    }

    fn read_double(&mut self) -> io::Result<f64> {
        let mut buf = [0u8; 8];
        self.stream.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn get_hardware_list(&mut self) -> io::Result<String> {
        self.send_command(Command::GetHardware)?;

        let mut reader = BufReader::new(&self.stream);
        let mut json = String::new();
        reader.read_line(&mut json)?;

        if json.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "No data received",
            ));
        }
        Ok(json.trim().to_string())
    }

    pub fn get_value(&mut self, index: i32) -> io::Result<f64> {
        self.send_command(Command::GetValue)?;
        self.send_int(index)?;
        self.read_double()
    }

    pub fn update(&mut self) -> io::Result<()> {
        self.send_command(Command::Update)
    }

    pub fn disconnect(mut self) -> io::Result<()> {
        let _ = self.send_command(Command::Shutdown);
        self.process_handle.kill()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hardware_list() {
        let mut helper = LhmHelper::connect().unwrap();
        assert!(!helper.get_hardware_list().unwrap().is_empty());
        helper.disconnect().unwrap();
    }

    #[test]
    fn test_get_value() {
        let mut helper = LhmHelper::connect().unwrap();
        let value = helper.get_value(0);
        assert!(value.is_ok());
        helper.disconnect().unwrap();
    }
}
