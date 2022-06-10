use amiquip::{Connection, ExchangeDeclareOptions, ExchangeType, Publish, Result};
use envconfig::Envconfig;
use log::{debug, info};
use tp2::messages::Message;
use tp2::post::PostIterator;
use tp2::{Config, POSTS_SOURCE_EXCHANGE_NAME};

fn main() -> Result<()> {
    let env_config = Config::init_from_env().unwrap();
    println!("Setting logger level: {}", env_config.logging_level);
    std::env::set_var("RUST_LOG", env_config.logging_level.clone());
    env_logger::init();
    let posts_file = envconfig::load_var_with_default("POSTS_FILE", None, "").unwrap();
    run_service(env_config, posts_file)
}

fn run_service(config: Config, posts_file: String) -> Result<()> {
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
        POSTS_SOURCE_EXCHANGE_NAME,
        exchange_options,
    )?;

    let posts = PostIterator::from_file(&posts_file);
    info!("Iterating posts");
    let published = posts
        .map(Message::FullPost)
        .flat_map(|message| {
            debug!("Publishing {:?}", message);
            bincode::serialize(&message)
        })
        .flat_map(|data| exchange.publish(Publish::new(&data, "")))
        .count();

    info!("Published {} comments", published);

    let data = bincode::serialize(&Message::EndOfStream).unwrap();
    exchange.publish(Publish::new(&data, ""))?;
    info!("Exit");
    connection.close()
}
