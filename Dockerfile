FROM rust:1.47

WORKDIR app

COPY . .

#use sqlx-data json for schema
ENV SQLX_OFFLINE true

RUN cargo build --release

ENV APP_ENVIRONMENT production

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]
