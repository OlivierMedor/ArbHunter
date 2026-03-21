use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    println!("QUICKNODE_WSS_URL: {:?}", env::var("QUICKNODE_WSS_URL"));
    println!("RPC_HTTP_URL: {:?}", env::var("RPC_HTTP_URL"));
    println!("QUICKNODE_HTTP_URL: {:?}", env::var("QUICKNODE_HTTP_URL"));
}
