use amiquip::{Connection, ExchangeDeclareOptions, ExchangeType, Publish, Result};
use envconfig::Envconfig;
use log::{debug, info};
use tp2::comment::CommentIterator;
use tp2::messages::Message;
use tp2::COMMENTS_SOURCE_EXCHANGE_NAME;

#[derive(Envconfig)]
struct Config {
    /// logger level: valid values: "DEBUG", "INFO", "WARN", "ERROR"
    #[envconfig(from = "LOGGING_LEVEL", default = "INFO")]
    logging_level: String,
    /// RabbitMQ host
    #[envconfig(from = "RABBITMQ_HOST", default = "localhost")]
    server_host: String,
    /// RabbitMQ port
    #[envconfig(from = "RABBITMQ_PORT", default = "5672")]
    server_port: String,
    /// RabbitMQ username
    #[envconfig(from = "RABBITMQ_USER", default = "guest")]
    user: String,
    /// RabbitMQ password
    #[envconfig(from = "RABBITMQ_PASS", default = "guest")]
    pass: String,
    /// Comments source file
    #[envconfig(
        from = "COMMENTS_FILE",
        default = "data/the-reddit-irl-dataset-comments-head.csv"
    )]
    comments_file: String,
}

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
    let exchange_options = ExchangeDeclareOptions {
        durable: true,
        ..ExchangeDeclareOptions::default()
    };
    let exchange = channel.exchange_declare(
        ExchangeType::Fanout,
        COMMENTS_SOURCE_EXCHANGE_NAME,
        exchange_options,
    )?;

    let comments = CommentIterator::from_file(&config.comments_file);
    info!("Iterating comments");
    comments
        .map(|comment| Message::FullComment(comment))
        .flat_map(|message| {
            debug!("Publishing {:?}", message);
            bincode::serialize(&message)
        })
        .for_each(|data| {
            exchange.publish(Publish::new(&data, "")).unwrap();
        });

    let data = bincode::serialize(&Message::EndOfStream).unwrap();
    exchange.publish(Publish::new(&data, ""))?;
    info!("Exit");
    connection.close()
}
