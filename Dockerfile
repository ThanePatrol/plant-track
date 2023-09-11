FROM rustlang/rust:nightly-buster

WORKDIR /usr/app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src
COPY index.html index.html

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
RUN trunk build --release

EXPOSE 8080
CMD ["trunk" "serve" "--address" "0.0.0.0"]
