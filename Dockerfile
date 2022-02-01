# Stolen from https://gist.github.com/PurpleBooth/ec81bad0a7b56ac767e0da09840f835a
FROM rust:alpine

WORKDIR /build
COPY . /build
ENV SQLX_OFFLINE true
RUN apk add musl-dev openssl-dev vips-dev
RUN cargo build --release
RUN cargo install sqlx-cli
RUN cp /usr/local/cargo/bin/sqlx /sqlx
RUN cp /build/migrations /migrations
RUN cp /build/target/release/emotes-rs /emotes-rs
RUN cp /build/emotes-rs-startup.sh /emotes-rs-startup.sh
CMD ["/emotes-rs-startup.sh"]
