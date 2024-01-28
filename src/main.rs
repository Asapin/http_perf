use dispatcher::Dispatcher;
use models::common::Config;
use request_generator::RequestGenerator;

mod dispatcher;
mod models;
mod request_generator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        total_number_of_connections: 5,
        number_of_requests: 20000,
    };
    let dispatcher = Dispatcher::new(&config, |idx| RequestGenerator::new(idx));
    dispatcher.run().await;
    Ok(())
}
