use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use envconfig::Envconfig;
use log::{debug, error, info};
use tp2::Config;
use tp2::messages::Message;

const N_RESULTS: usize = 3;

fn main() -> Result<()> {
    let env_config = Config::init_from_env().unwrap();
    println!("Setting logger level: {}", env_config.logging_level);
    std::env::set_var("RUST_LOG", env_config.logging_level.clone());
    env_logger::init();
    run_service(env_config)
}

fn run_service(config: Config) -> Result<()> {
    let host_addr = format!(
        "amqp://{}:{}@{}:{}",
        config.user, config.pass, config.server_host, config.server_port
    );
    debug!("Connecting to: {}", host_addr);

    let mut connection = Connection::insecure_open(&host_addr)?;
    let channel = connection.open_channel(None)?;

    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };
    let queue = channel.queue_declare("results", options)?;

    // Query results
    let consumer = queue.consume(ConsumerOptions::default())?;
    for consumer_message in consumer.receiver().iter().take(N_RESULTS) {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::PostScoreMean(mean)) => {
                    info!("Post score mean: {}", mean);
                }
                _ => {
                    error!("Invalid message arrived");
                }
            }
            consumer.ack(delivery)?;
        }
    }
    info!("Exit");
    connection.close()
}
