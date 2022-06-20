use amiquip::{
    Channel, Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result,
};
use envconfig::Envconfig;
use log::{debug, error, info};
use tp2::messages::Message;
use tp2::{
    Config, POST_EXTRACTED_URL_QUEUE_NAME, POST_SENTIMENT_MEAN_QUEUE_NAME, RESULTS_QUEUE_NAME,
};

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

    // Score producer
    let exchange = Exchange::direct(&channel);

    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };

    let best_meme_id = get_best_meme_id(&channel)?;
    info!("Got best meme ID: {}", best_meme_id);

    let queue = channel.queue_declare(POST_EXTRACTED_URL_QUEUE_NAME, options)?;

    // Query results
    let consumer = queue.consume(ConsumerOptions::default())?;

    let mut meme_url = "Not found".to_string();

    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::EndOfStream) => {
                    consumer.ack(delivery)?;
                    break;
                }
                Ok(Message::PostUrl(id, url)) => {
                    if id == best_meme_id {
                        info!("Best meme with url: {:?} {:?}", id, url);
                        meme_url = url
                    }
                }
                _ => {
                    error!("Invalid message arrived");
                }
            }
            consumer.ack(delivery)?;
        }
    }
    debug!("Sending best meme url: {}", meme_url);
    let body = bincode::serialize(&Message::PostUrl(best_meme_id, meme_url)).unwrap();
    exchange.publish(Publish::new(&body, RESULTS_QUEUE_NAME))?;
    info!("Exit");
    connection.close()
}

// Should I use a heap of best memes ids in case the best one is missing?
fn get_best_meme_id(channel: &Channel) -> Result<String> {
    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };
    let queue = channel.queue_declare(POST_SENTIMENT_MEAN_QUEUE_NAME, options)?;
    // Query results
    let consumer = queue.consume(ConsumerOptions::default())?;
    let mut best_meme_id_sentiment = ("".to_string(), f32::MIN);
    if let Some(consumer_message) = consumer.receiver().iter().next() {
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
    info!("Best meme sentiment: {:?}", best_meme_id_sentiment);
    Ok(best_meme_id_sentiment.0)
}
