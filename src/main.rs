use std::{
    io::{self, BufRead},
    process::{Command, Stdio},
    sync::mpsc::{self, Sender},
    thread,
};
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
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let tx_stderr = tx.clone();
    thread::spawn(move || {
        let err_reader = io::BufReader::new(stderr);
        for err_line in err_reader.lines() {
            let err_line = err_line.unwrap();
            tx_stderr
                .send(format!("{}:{}", command.name, err_line))
                .unwrap();
        }
    });

    let tx_stdout = tx.clone();
    thread::spawn(move || {
        let reader = io::BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.unwrap();
            tx_stdout.send(format!("{}:{}", command.name, line)).unwrap();
        }
    });
    Ok(())
}
