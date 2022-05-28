FROM rust:latest AS builder
LABEL intermediateStageToBeDeleted=true

COPY src build/src
COPY Cargo.toml Cargo.lock build/
RUN cd build && ls &&cargo build --release

FROM ubuntu:latest

RUN apt update &&\
  apt install -y libssl1.1

COPY --from=builder /build/target/release/post_producer /usr/bin/
COPY --from=builder /build/target/release/comment_producer /usr/bin/
COPY --from=builder /build/target/release/mean_calculator /usr/bin/
COPY --from=builder /build/target/release/score_extractor /usr/bin/

