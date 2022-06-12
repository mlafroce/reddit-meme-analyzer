use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions, Result,
};
use envconfig::Envconfig;
use log::{debug, error, info};
use std::collections::HashMap;
use tp2::messages::Message;
use tp2::{Config, FILTERED_POST_ID_SENTIMENT_QUEUE_NAME, POST_SENTIMENT_MEAN_QUEUE_NAME};

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

    // Post consumer
    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };
    let queue = channel.queue_declare(FILTERED_POST_ID_SENTIMENT_QUEUE_NAME, options)?;

    // Score producer
    let exchange = Exchange::direct(&channel);

    let consumer = queue.consume(ConsumerOptions::default())?;
    // postId -> (sentiment_sum, count)
    let mut post_sentiments_map = HashMap::<String, (f32, i32)>::new();
    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::EndOfStream) => {
                    let post_sentiment = get_highest_post_sentiment(&post_sentiments_map);
                    let body = bincode::serialize(&Message::PostIdSentiment(
                        post_sentiment.0,
                        post_sentiment.1,
                    ))
                    .unwrap();
                    exchange.publish(Publish::new(&body, POST_SENTIMENT_MEAN_QUEUE_NAME))?;
                    consumer.ack(delivery)?;
                    break;
                }
                Ok(Message::PostIdSentiment(post_id, sentiment)) => {
                    let value = post_sentiments_map.entry(post_id).or_insert((0.0, 0));
                    value.0 += sentiment;
                    value.1 += 1;
                }
                _ => {
                    error!("Invalid message arrived");
                }
            };
            consumer.ack(delivery)?;
        }
    }
    info!("Exit");
    connection.close()
}

fn get_highest_post_sentiment(sentiment_map: &HashMap<String, (f32, i32)>) -> (String, f32) {
    let mut highest_id = "".to_string();
    let mut highest = -1.0;
    for (id, sentiment) in sentiment_map.iter() {
        let sentiment_mean = sentiment.0 / sentiment.1 as f32;
        if sentiment_mean > highest {
            highest = sentiment_mean;
            highest_id = id.clone();
        }
    }
    (highest_id, highest)
}
