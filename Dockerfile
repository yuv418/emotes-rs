# Stolen from https://gist.github.com/PurpleBooth/ec81bad0a7b56ac767e0da09840f835a
FROM rust:alpine

WORKDIR /build
COPY . /build
ENV SQLX_OFFLINE true
RUN apk add musl-dev openssl-dev
RUN cargo build --release
RUN cargo install sqlx-cli

FROM busybox
COPY --from=0 /usr/local/cargo/bin/sqlx /sqlx
COPY --from=0 /build/migrations /migrations
COPY --from=0 /build/target/release/attendance-rs /attendance-rs
COPY --from=0 /build/attendance-rs-startup.sh /attendance-rs-startup.sh
CMD ["/attendance-rs-startup.sh"]
