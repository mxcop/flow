use std::{error::Error};

mod bootstrap_node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    //let args: Vec<String> = env::args().collect();
    //let is_bootstrap = args.len() > 1 && args.clone()[1] == "--bootstrap";

    // Create a new node.
    bootstrap_node::create().await
}
