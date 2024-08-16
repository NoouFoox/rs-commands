use std::{
    sync::mpsc::{self},
    thread,
};

use commands::{new_command, ExcCommand};

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
