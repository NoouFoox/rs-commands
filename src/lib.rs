use std::{
    io::{self, BufRead},
    process::{ChildStderr, ChildStdout, Command, Stdio},
    sync::mpsc::Sender,
    thread,
};

#[derive(Clone)]
pub struct ExcCommand {
    pub name: &'static str,
    pub content: &'static str,
}
pub fn new_command(command: ExcCommand, tx: Sender<String>) -> Result<(), io::Error> {
    println!("{}", command.content);
    let mut child = Command::new("sh")
        .arg("-c")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(command.content)
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
