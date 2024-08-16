use std::{
    sync::mpsc::{self},
    thread,
};

use commands::{new_command, read_config, ExcCommand};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "config.json";
    let config = read_config(config_path)?;
    let mut commands: Vec<ExcCommand> = vec![];
    if let Value::Object(map) = config {
        for (key, value) in map {
            let command = ExcCommand {
                name: key,
                content: value.as_str().unwrap_or("").to_string(),
            };
            commands.push(command)
        }
    }
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
    Ok(())
}
