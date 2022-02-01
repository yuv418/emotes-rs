# Stolen from https://gist.github.com/PurpleBooth/ec81bad0a7b56ac767e0da09840f835a
FROM rnestler/archlinux-rust

WORKDIR /build
COPY . /build
ENV SQLX_OFFLINE true
RUN pacman -Sy --noconfirm base-devel libvips
RUN cargo build --release
RUN cargo install sqlx-cli

RUN cp /root/.cargo/bin/sqlx /sqlx
RUN cp -r /build/migrations /migrations
RUN cp /build/target/release/emotes-rs /emotes-rs
RUN cp /build/emotes-rs-startup.sh /emotes-rs-startup.sh
CMD ["/emotes-rs-startup.sh"]
