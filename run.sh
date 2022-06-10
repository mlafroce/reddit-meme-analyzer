export LOGGING_LEVEL=info
export POSTS_FILE=data/the-reddit-irl-dataset-posts-head.csv
export COMMENTS_FILE=data/the-reddit-irl-dataset-comments-head.csv

docker start tp2_rabbitmq_1
sleep 5
bash scripts.sh
RABBITMQ_HOST=172.21.0.2 cargo run --bin results_consumer &
CONSUMER_PID=$!
RABBITMQ_HOST=172.21.0.2 cargo run --bin mean_calculator &
RABBITMQ_HOST=172.21.0.2 cargo run --bin score_extractor &

RABBITMQ_HOST=172.21.0.2 cargo run --bin comment_sentiment_extractor &
RABBITMQ_HOST=172.21.0.2 cargo run --bin post_sentiment_calculator &
RABBITMQ_HOST=172.21.0.2 cargo run --bin url_extractor &
RABBITMQ_HOST=172.21.0.2 cargo run --bin best_meme_filter &

RABBITMQ_HOST=172.21.0.2 cargo run --bin comment_college_filter &
RABBITMQ_HOST=172.21.0.2 cargo run --bin post_college_filter &
RABBITMQ_HOST=172.21.0.2 cargo run --bin post_average_filter &

RABBITMQ_HOST=172.21.0.2 cargo run --bin post_producer &
RABBITMQ_HOST=172.21.0.2 cargo run --bin comment_producer &

wait $CONSUMER_PID
