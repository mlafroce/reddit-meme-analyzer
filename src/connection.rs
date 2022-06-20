use amiquip::{Channel, Connection, Consumer, ConsumerOptions, Exchange, ExchangeDeclareOptions, ExchangeType, Publish, QueueDeclareOptions, Result};
use log::{debug, error};
use crate::Config;
use crate::messages::Message;

pub struct RabbitConnection {
    connection: Connection,
    channel: Channel,
}

impl RabbitConnection {
    pub fn new(config: Config) -> Result<Self> {
        let host_addr = format!(
            "amqp://{}:{}@{}:{}",
            config.user, config.pass, config.server_host, config.server_port
        );
        debug!("Connecting to: {}", host_addr);
        let mut connection = Connection::insecure_open(&host_addr)?;
        let channel = connection.open_channel(None)?;
        Ok(Self {
            connection,
            channel,
        })
    }

    pub fn get_consumer(&self, channel_id: &str) -> Result<Consumer> {
        let options = QueueDeclareOptions {
            auto_delete: false,
            ..QueueDeclareOptions::default()
        };
        let queue = self.channel.queue_declare(channel_id, options)?;
        queue.consume(ConsumerOptions::default())
    }

    pub fn get_direct_exchange(&self) -> Exchange {
        Exchange::direct(&self.channel)
    }

    pub fn get_fanout_exchange(&mut self, exchange: &str) -> Result<Exchange> {
        let exchange_options = ExchangeDeclareOptions {
            durable: true,
            ..ExchangeDeclareOptions::default()
        };
        self.channel
            .exchange_declare(ExchangeType::Fanout, exchange, exchange_options)
    }

    pub fn close(self) -> Result<()> {
        self.connection.close()
    }
}

pub struct BinaryExchange<'a> {
    exchange: Exchange<'a>,
    output_key: String,
    producers: usize,
    consumers: usize,
    finished_producers: usize,
    eos_message: Message,
}

impl<'a> BinaryExchange<'a> {
    pub fn new(
        exchange: Exchange<'a>,
        output_key: Option<String>,
        producers: usize,
        consumers: usize,
    ) -> Self {
        let output_key = output_key.unwrap_or("".to_owned());
        let eos_message = Message::EndOfStream;
        let finished_producers = 0;
        Self {
            exchange,
            output_key,
            producers,
            consumers,
            finished_producers,
            eos_message,
        }
    }

    pub fn send<T>(&self, message: &T) -> Result<()>
        where
            T: serde::Serialize,
    {
        let body = bincode::serialize(message).unwrap();
        self.exchange.publish(Publish::new(&body, &self.output_key))
    }

    /// Call when an end of stream arrives. If no producers are left, notify consumers about EOS
    /// Returns true if finished, false otherwise
    pub fn producer_ended(&mut self) -> Result<bool> {
        if self.finished_producers < self.producers - 1 {
            self.finished_producers += 1;
            Ok(false)
        } else if self.finished_producers == self.producers - 1 {
            self.finished_producers += 1;
            for _ in 0..self.consumers {
                self.send(&self.eos_message)?;
            }
            Ok(true)
        } else {
            error!("Received extra End Of stream");
            Ok(true)
        }
    }
}