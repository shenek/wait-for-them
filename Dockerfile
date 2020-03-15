FROM rust

WORKDIR /root/build

COPY . .

RUN cargo install --path .
