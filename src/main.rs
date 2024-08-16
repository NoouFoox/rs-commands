use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};
use commands::{new_command, read_config, ExcCommand, SharedState};
use serde_json::Value;
use warp::Filter;
use warp::http::Method;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "config.json";
    let config = read_config(config_path)?;
    let mut commands: Vec<ExcCommand> = vec![];
    if let Value::Object(map) = config {
        for (key, value) in map {
            let command = ExcCommand {
                name: key.clone(),
                content: value.as_str().unwrap_or("").to_string(),
            };
            commands.push(command);
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let output_store: SharedState = Arc::new(Mutex::new(HashMap::new()));

    for command in commands {
        let output_store = output_store.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = new_command(command, tx, output_store);
        });
    }
    
    let output_store = warp::any().map(move || output_store.clone());

    let get_output = warp::path!("command" / String)
        .and(output_store)
        .map(|name: String, output_store: SharedState| {
            let output_store = output_store.lock().unwrap();
            match output_store.get(&name) {
                Some(output) => warp::reply::html(output.join("\n")),
                None => warp::reply::html("No output found".to_string()),
            }
        });
    // CORS 过滤器
    let cors = warp::cors()
        .allow_any_origin() 
        .allow_methods(&[Method::GET, Method::POST])
        .allow_headers(vec!["Content-Type", "Authorization"]);
    let routes = warp::get().and(get_output).with(cors);
    tokio::spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    });
    for received in rx {
        println!("{}", received);
    }

    Ok(())
}
