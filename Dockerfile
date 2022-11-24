FROM ubuntu:22.04

RUN apt update -y
RUN apt install -y ca-certificates

RUN mkdir /app

COPY ./target/release/telegram-lawn-bot /app/telegram-lawn-bot
COPY ./secret.toml /app/secret.toml

WORKDIR /app

ENTRYPOINT ["./telegram-lawn-bot", "-s", "./secret.toml"]