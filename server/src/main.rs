use core::panic;

#[tokio::main]
async fn main() {
    panic!("This is a test to ensure that uptime monitoring is working correctly");
    server::run().await
}
