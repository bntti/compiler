FROM rust:1-alpine

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache build-base musl-dev gcc

COPY Cargo.toml Cargo.lock ./

COPY src src/
RUN cargo install --path .

EXPOSE 3000

CMD ["compiler", "--command", "serve", "--host", "0.0.0.0"]
