use amiquip::{
    Channel, Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result,
};
use envconfig::Envconfig;
use log::{debug, error, info};
use std::collections::HashSet;
use tp2::messages::Message;
use tp2::{
    Config, FILTERED_POST_ID_SENTIMENT_QUEUE_NAME, POST_ID_SENTIMENT_QUEUE_NAME,
    POST_ID_WITH_URL_QUEUE_NAME,
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
    info!("Getting post ids with url");
    let ids = get_posts_ids_with_url(&channel)?;
    info!("Filtering sentiments with url");
    filter_posts_with_urls(channel, ids)?;
    info!("Exit");
    connection.close()
}

fn get_posts_ids_with_url(channel: &Channel) -> Result<HashSet<String>> {
    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };
    let queue = channel.queue_declare(POST_ID_WITH_URL_QUEUE_NAME, options)?;
    let consumer = queue.consume(ConsumerOptions::default())?;
    let mut ids = HashSet::new();
    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::EndOfStream) => {
                    consumer.ack(delivery)?;
                    break;
                }
                Ok(Message::PostId(id)) => {
                    ids.insert(id);
                }
                _ => {
                    error!("Invalid message arrived");
                }
            };
            consumer.ack(delivery)?;
        }
    }
    Ok(ids)
}

fn filter_posts_with_urls(channel: Channel, ids: HashSet<String>) -> Result<HashSet<String>> {
    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };
    let queue = channel.queue_declare(POST_ID_SENTIMENT_QUEUE_NAME, options)?;
    let consumer = queue.consume(ConsumerOptions::default())?;
    let exchange = Exchange::direct(&channel);

    let mut count = 0;
    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::EndOfStream) => {
                    exchange.publish(Publish::new(
                        &bincode::serialize(&Message::EndOfStream).unwrap(),
                        FILTERED_POST_ID_SENTIMENT_QUEUE_NAME,
                    ))?;
                    consumer.ack(delivery)?;
                    break;
                }
                Ok(Message::PostIdSentiment(post_id, sentiment)) => {
                    count += 1;
                    if ids.contains(&post_id) {
                        let msg = Message::PostIdSentiment(post_id, sentiment);
                        exchange.publish(Publish::new(
                            &bincode::serialize(&msg).unwrap(),
                            FILTERED_POST_ID_SENTIMENT_QUEUE_NAME,
                        ))?;
                    }
                }
                _ => {
                    error!("Invalid message arrived");
                }
            };
            consumer.ack(delivery)?;
        }
    }
    info!("Filtered {} posts", count);
    Ok(ids)
}
