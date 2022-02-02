use std::{error::Error};

mod node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    node::create().await
}