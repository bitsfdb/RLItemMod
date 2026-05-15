mod engine;

use std::env;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: velocity-api <service> [json_body]");
        return Ok(());
    }

    let service = &args[1];
    let body = if args.len() > 2 {
        serde_json::from_str(&args[2]).map_err(|e| e.to_string())?
    } else {
        json!({})
    };

    println!("Calling {}...", service);
    let result: serde_json::Value = engine::psynet::call_rpc(service, &body, None, None).await?;
    println!("{}", serde_json::to_string_pretty(&result).unwrap());

    Ok(())
}
