use amiquip::{
    Connection, Consumer, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result,
};
use envconfig::Envconfig;
use log::{debug, error, info, warn};
use tp2::Config;
use tp2::messages::Message;


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

    let best_meme_id = get_best_meme_id(&consumer)?;

    let mut best_meme_found = false;
    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::PostUrl(id, sentiment)) => {
                    if id == best_meme_id {
                        info!("Best meme with sentiment: {:?} {:?}", id, sentiment);
                        best_meme_found = true;
                    }
                }
                Ok(_) => {
                    // Todo Notify invalid messages?
                    error!("Invalid message arrived");
                }
                Err(_) => {
                    warn!("Consumer ended unexpectedly: {:?}", delivery);
                }
            }
            consumer.ack(delivery)?;
        }
    }
    if !best_meme_found {
        info!("Best meme (id: {}) was not found", best_meme_id);
    }
    info!("Exit");
    connection.close()
}

// Should I use a heap of best memes ids in case the best one is missing?
fn get_best_meme_id(consumer: &Consumer) -> Result<String> {
    let mut best_meme_id_sentiment = ("".to_string(), f32::MIN);
    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::PostIdSentiment(id, sentiment)) => {
                    if sentiment > best_meme_id_sentiment.1 {
                        best_meme_id_sentiment = (id, sentiment);
                    }
                }
                _ => {
                    error!("Invalid message arrived");
                }
            }
            consumer.ack(delivery)?;
        }
    }
    Ok(best_meme_id_sentiment.0)
}
