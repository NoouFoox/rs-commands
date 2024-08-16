use std::{
    fs::File, io::{self, BufRead, BufReader}, path::Path, process::{ChildStderr, ChildStdout, Command, Stdio}, sync::mpsc::Sender, thread
};

use serde_json::Value;

#[derive(Clone)]
pub struct ExcCommand {
    pub name: String,
    pub content: String,
}
pub fn new_command(command: ExcCommand, tx: Sender<String>) -> Result<(), io::Error> {
    println!("{}", command.content);
    #[cfg(target_os = "windows")]
    let mut child = Command::new("cmd")
        .args(&["/C", &command.content])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let mut child = Command::new("sh")
        .arg("-c")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&command.content)
        .spawn()?;
    let stdout: ChildStdout = child.stdout.take().unwrap();
    let stderr: ChildStderr = child.stderr.take().unwrap();
    send_line(&tx, command.clone(), stderr);
    send_line(&tx, command, stdout);
    Ok(())
}

pub fn send_line<T>(tx: &Sender<String>, command: ExcCommand, stream: T)
where
    T: io::Read + Send + 'static,
{
    let tx = tx.clone();
    thread::spawn(move || {
        let reader = io::BufReader::new(stream);
        for line in reader.lines() {
            let line = line.unwrap();
            tx.send(format!("{}:{}", command.name, line)).unwrap();
        }
    });
}
pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Value, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
