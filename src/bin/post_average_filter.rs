use amiquip::{
    Channel, Connection, ConsumerMessage, ConsumerOptions, Error, Exchange, Publish,
    QueueDeclareOptions, Result,
};
use envconfig::Envconfig;
use log::{debug, error, info};
use tp2::messages::Message;
use tp2::{
    Config, POST_COLLEGE_QUEUE_NAME, POST_SCORE_AVERAGE_QUEUE_NAME, POST_URL_AVERAGE_QUEUE_NAME,
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

    let score_average = get_score_average(&channel)?;

    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };

    let queue = channel.queue_declare(POST_COLLEGE_QUEUE_NAME, options)?;

    let exchange = Exchange::direct(&channel);

    // Query results
    let consumer = queue.consume(ConsumerOptions::default())?;

    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            let message = bincode::deserialize::<Message>(&delivery.body);
            match message {
                Ok(Message::FullPost(post)) => {
                    if post.score as f32 > score_average && post.url.starts_with("https") {
                        let msg = Message::PostUrl(post.id, post.url);
                        let body = bincode::serialize(&msg).unwrap();
                        exchange.publish(Publish::new(&body, POST_URL_AVERAGE_QUEUE_NAME))?;
                    }
                }
                Ok(Message::EndOfStream) => {
                    exchange.publish(Publish::new(
                        &bincode::serialize(&Message::EndOfStream).unwrap(),
                        POST_URL_AVERAGE_QUEUE_NAME,
                    ))?;
                    consumer.ack(delivery)?;
                    break;
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

// Should I use a heap of best memes ids in case the best one is missing?
fn get_score_average(channel: &Channel) -> Result<f32> {
    let options = QueueDeclareOptions {
        auto_delete: false,
        ..QueueDeclareOptions::default()
    };
    let queue = channel.queue_declare(POST_SCORE_AVERAGE_QUEUE_NAME, options)?;
    // Query results
    let consumer = queue.consume(ConsumerOptions::default())?;

    for consumer_message in consumer.receiver().iter() {
        if let ConsumerMessage::Delivery(delivery) = consumer_message {
            match bincode::deserialize::<Message>(&delivery.body) {
                Ok(Message::PostScoreMean(mean)) => {
                    consumer.ack(delivery)?;
                    return Ok(mean);
                }
                _ => {
                    error!("Invalid message arrived");
                }
            }
            consumer.ack(delivery)?;
        }
    }
    Err(Error::ClientException)
}
