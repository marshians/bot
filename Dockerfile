FROM rust

# RUN apt-get update -y && apt-get install -y musl-tools

COPY . /src

WORKDIR /src

RUN cargo build --release

EXPOSE 8000

ENV ROCKET_ADDRESS "0.0.0.0"

ENTRYPOINT ["/src/target/release/http"]
