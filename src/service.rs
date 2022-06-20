use crate::connection::{BinaryExchange, RabbitConnection};
use crate::messages::Message;
use crate::Config;
use amiquip::{ConsumerMessage, Result};
use log::{error, info, warn};

pub trait RabbitService {
    fn process_message(&mut self, message: Message, bin_exchange: &BinaryExchange) -> Result<()>;

    fn on_stream_finished(&self, bin_exchange: &BinaryExchange) -> Result<()>;

    fn run(&mut self, config: Config, consumer: &str, output_key: Option<String>) -> Result<()> {
        let consumers = str::parse::<usize>(&config.consumers).unwrap();
        let producers = str::parse::<usize>(&config.producers).unwrap();
        let connection = RabbitConnection::new(&config)?;
        {
            let exchange = connection.get_direct_exchange();
            let consumer = connection.get_consumer(consumer)?;
            let mut bin_exchange = BinaryExchange::new(exchange, output_key, producers, consumers);

            for consumer_message in consumer.receiver().iter() {
                if let ConsumerMessage::Delivery(delivery) = consumer_message {
                    let message = bincode::deserialize::<Message>(&delivery.body);
                    match message {
                        Ok(Message::EndOfStream) => {
                            let stream_finished = bin_exchange.producer_ended()?;
                            if stream_finished {
                                self.on_stream_finished(&bin_exchange)?;
                                consumer.ack(delivery)?;
                                break;
                            } else {
                                consumer.ack(delivery)?;
                                continue;
                            }
                        }
                        Ok(message) => {
                            self.process_message(message, &bin_exchange)?;
                        }
                        Err(_) => {
                            error!("Invalid message arrived");
                        }
                    };
                    consumer.ack(delivery)?;
                } else {
                    warn!("RabbitService delivery failed");
                }
            }
        }
        info!("Exit");
        connection.close()?;
        Ok(())
    }
}
