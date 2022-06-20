use amiquip::{ConsumerMessage, Result};
use log::{error, info, warn};
use std::collections::HashSet;
use tp2::connection::{BinaryExchange, RabbitConnection};
use tp2::messages::Message;
use tp2::service::{init, RabbitService};
use tp2::{Config, POST_ID_COLLEGE_QUEUE_NAME, POST_URL_AVERAGE_QUEUE_NAME, RESULTS_QUEUE_NAME};

fn main() -> Result<()> {
    let env_config = init();
    run_service(env_config)
}

fn run_service(config: Config) -> Result<()> {
    info!("Getting college post ids");
    let ids = get_college_posts_ids(&config)?;
    info!("Filtering college posts");
    let mut service = PostCollegeFilter { ids };
    service.run(
        config,
        POST_URL_AVERAGE_QUEUE_NAME,
        Some(RESULTS_QUEUE_NAME.to_string()),
    )
}

struct PostCollegeFilter {
    ids: HashSet<String>,
}

impl RabbitService for PostCollegeFilter {
    fn process_message(&mut self, message: Message, exchange: &BinaryExchange) -> Result<()> {
        match message {
            Message::PostUrl(id, url) => {
                if self.ids.contains(&id) {
                    let msg = Message::CollegePostUrl(url);
                    exchange.send(&msg)?;
                }
            }
            _ => {
                warn!("Invalid message arrived");
            }
        }
        Ok(())
    }
}

fn get_college_posts_ids(config: &Config) -> Result<HashSet<String>> {
    let connection = RabbitConnection::new(config)?;
    let mut ids = HashSet::new();
    {
        let consumer = connection.get_consumer(POST_ID_COLLEGE_QUEUE_NAME)?;
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
    }
    connection.close()?;
    Ok(ids)
}
