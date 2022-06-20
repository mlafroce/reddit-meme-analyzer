use amiquip::{ExchangeType, Publish, Result};
use envconfig::Envconfig;
use log::{debug, info};
use tp2::comment::CommentIterator;
use tp2::connection::RabbitConnection;
use tp2::messages::Message;
use tp2::{Config, COMMENTS_SOURCE_EXCHANGE_NAME};

fn main() -> Result<()> {
    let env_config = Config::init_from_env().unwrap();
    println!("Setting logger level: {}", env_config.logging_level);
    std::env::set_var("RUST_LOG", env_config.logging_level.clone());
    env_logger::init();
    let comments_file = envconfig::load_var_with_default(
        "COMMENTS_FILE",
        None,
        "data/the-reddit-irl-dataset-comments.csv",
    )
    .unwrap();
    run_service(env_config, comments_file)
}

fn run_service(config: Config, comments_file: String) -> Result<()> {
    let connection = RabbitConnection::new(&config)?;
    let exchange =
        connection.get_named_exchange(COMMENTS_SOURCE_EXCHANGE_NAME, ExchangeType::Fanout)?;
    let comments = CommentIterator::from_file(&comments_file);
    info!("Iterating comments");
    let published = comments
        .map(Message::FullComment)
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
