use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex, mpsc::Sender},
    thread,
};
use serde_json::Value;

#[derive(Clone)]
pub struct ExcCommand {
    pub name: String,
    pub content: String,
}

pub type SharedState = Arc<Mutex<HashMap<String, Vec<String>>>>;

pub fn new_command(command: ExcCommand, tx: Sender<String>, output_store: SharedState) -> Result<(), io::Error> {
    println!("{}", command.content);

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let mut child = Command::new("sh")
        .arg("-c")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&command.content)
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        send_line(&tx, command.clone(), stdout, output_store.clone());
    }

    if let Some(stderr) = child.stderr.take() {
        send_line(&tx, command, stderr, output_store);
    }

    Ok(())
}

pub fn send_line<T>(tx: &Sender<String>, command: ExcCommand, stream: T, output_store: SharedState)
where
    T: io::Read + Send + 'static,
{
    let tx = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    tx.send(line.clone()).unwrap();  // 仅发送行内容，不附加命令标识符

                    // 将输出存储在共享状态中
                    let mut output_store = output_store.lock().unwrap();
                    output_store.entry(command.name.clone()).or_insert(Vec::new()).push(line);
                }
                Err(e) => {
                    let error_line = format!("Error reading line: {}", e);
                    tx.send(error_line.clone()).unwrap();

                    // 将错误信息存储在共享状态中
                    let mut output_store = output_store.lock().unwrap();
                    output_store.entry(command.name.clone()).or_insert(Vec::new()).push(error_line);
                }
            }
        }
    });
}
pub fn read_config<P: AsRef<std::path::Path>>(path: P) -> Result<Value, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
