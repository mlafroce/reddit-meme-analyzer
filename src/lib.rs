use envconfig::Envconfig;

pub mod comment;
pub mod messages;
pub mod post;

#[derive(Envconfig)]
pub struct Config {
    /// logger level: valid values: "DEBUG", "INFO", "WARN", "ERROR"
    #[envconfig(from = "LOGGING_LEVEL", default = "INFO")]
    pub logging_level: String,
    /// RabbitMQ host
    #[envconfig(from = "RABBITMQ_HOST", default = "localhost")]
    pub server_host: String,
    /// RabbitMQ port
    #[envconfig(from = "RABBITMQ_PORT", default = "5672")]
    pub server_port: String,
    /// RabbitMQ username
    #[envconfig(from = "RABBITMQ_USER", default = "guest")]
    pub user: String,
    /// RabbitMQ password
    #[envconfig(from = "RABBITMQ_PASS", default = "guest")]
    pub pass: String,
}

/// Exchange with full posts
pub const POSTS_SOURCE_EXCHANGE_NAME: &str = "tp2.posts";
/// Exchange with full posts
pub const COMMENTS_SOURCE_EXCHANGE_NAME: &str = "tp2.comments";
/// Queue with full posts for mean score calculations
pub const POST_SCORES_QUEUE_NAME: &str = "tp2.posts.score_src";
/// Queue with full posts for college memes extraction
pub const POST_COLLEGE_QUEUE_NAME: &str = "tp2.posts.college_src";
/// Queue with full posts for college memes extraction
pub const POST_URL_QUEUE_NAME: &str = "tp2.posts.url_src";
/// Queue with posts urls with above average score
pub const POST_URL_AVERAGE_QUEUE_NAME: &str = "tp2.posts.above_average";
/// Queue with posts urls for best meme
pub const POST_EXTRACTED_URL_QUEUE_NAME: &str = "tp2.posts.urls";
/// Input for the mean calculator
pub const POST_SCORE_MEAN_QUEUE_NAME: &str = "tp2.posts.mean";
/// Input of college posts filter. Output of the mean calculator
pub const POST_SCORE_AVERAGE_QUEUE_NAME: &str = "tp2.posts.mean.result";
/// Input of comment sentiment extractor
pub const COMMENT_SENTIMENT_QUEUE_NAME: &str = "tp2.comments.sentiment_src";
/// Input of post sentiment calculator. Output of comment sentiment extractor.
pub const POST_ID_SENTIMENT_QUEUE_NAME: &str = "tp2.posts.sentiment";
/// Output of post sentiment calculator
pub const POST_SENTIMENT_MEAN_QUEUE_NAME: &str = "tp2.posts.sentiment.best";
/// Input of college comment filter
pub const COMMENT_COLLEGE_QUEUE_NAME: &str = "tp2.comments.college_src";
/// Input of college posts filter. Output of college comment filter.
pub const POST_ID_COLLEGE_QUEUE_NAME: &str = "tp2.posts.college_id";
/// Sink of college posts filter.
pub const COLLEGE_MEME_SINK_QUEUE_NAME: &str = "tp2.posts.college.sink";
/// Sink of mean calculator.
pub const MEAN_SCORE_SINK_QUEUE_NAME: &str = "tp2.posts.mean.sink";
/// Sink of best meme filter.
pub const BEST_MEME_SINK_QUEUE_NAME: &str = "tp2.posts.best.sink";
/// Results queue
pub const RESULTS_QUEUE_NAME: &str = "tp2.results";