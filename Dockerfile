FROM rust:1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY examples ./examples
COPY tests ./tests
RUN cargo build --release

FROM debian:bookworm-slim

RUN useradd --create-home --shell /usr/sbin/nologin digipin
COPY --from=builder /app/target/release/digipin /usr/local/bin/digipin
USER digipin

EXPOSE 8080
ENTRYPOINT ["digipin"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8080"]
