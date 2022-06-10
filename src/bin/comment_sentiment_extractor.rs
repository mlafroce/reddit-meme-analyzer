use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions, Result,
};
use envconfig::Envconfig;
use log::{debug, error, info};
use tp2::messages::Message;
use tp2::{Config, COMMENT_SENTIMENT_QUEUE_NAME, POST_ID_SENTIMENT_QUEUE_NAME};

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
    let queue = channel.queue_declare(COMMENT_SENTIMENT_QUEUE_NAME, options)?;

    // Score producer
    let exchange = Exchange::direct(&channel);

    let consumer = queue.consume(ConsumerOptions::default())?;
    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::EndOfStream) => {
                    exchange.publish(Publish::new(
                        &bincode::serialize(&Message::EndOfStream).unwrap(),
                        POST_ID_SENTIMENT_QUEUE_NAME,
                    ))?;
                    consumer.ack(delivery)?;
                    break;
                }
                Ok(Message::FullComment(comment)) => {
                    if let Some(post_id) = comment.parse_post_id() {
                        let msg = Message::PostIdSentiment(post_id, comment.sentiment);
                        debug!("Publishing {:?}", msg);
                        exchange.publish(Publish::new(
                            &bincode::serialize(&msg).unwrap(),
                            POST_ID_SENTIMENT_QUEUE_NAME,
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
    info!("Exit");
    connection.close()
}
