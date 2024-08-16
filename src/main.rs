use std::{
    io::{self, BufRead},
    process::{ChildStderr, ChildStdout, Command, Stdio},
    sync::mpsc::{self, Sender},
    thread,
};
#[derive(Clone)]
struct ExcCommand {
    name: &'static str,
    content: &'static str,
}

fn main() {
    let commands: Vec<ExcCommand> = vec![
        ExcCommand {
            name: "agent",
            content: "cd /Volumes/work/code/agent.kgd.ltd && pnpm dev:agent",
        },
        ExcCommand {
            name: "app",
            content: "cd /Volumes/work/code/agent.kgd.ltd && pnpm dev:app",
        },
    ];
    let (tx, rx) = mpsc::channel();
    for command in commands {
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = new_command(command, tx);
        });
    }
    drop(tx);
    for received in rx {
        println!("{}", received);
    }
}
fn new_command(command: ExcCommand, tx: Sender<String>) -> Result<(), io::Error> {
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
fn send_line<T>(tx: &Sender<String>, command: ExcCommand, stream: T)
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
