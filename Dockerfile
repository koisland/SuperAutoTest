FROM rust:1.49-slim-buster

RUN apt-get update -y && \
    apt-get install -y pkg-config libssl-dev libsqlite3-dev

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM rust:1.49-slim-buster
RUN apt-get update -y && \
    apt-get install -y libssl-dev libsqlite3-dev

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/sapdb .
COPY --from=builder /usr/src/app/config ./config
COPY --from=builder /usr/src/app/src ./src
COPY --from=builder /usr/src/app/Rocket.toml .

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

CMD ["./sapdb"]
