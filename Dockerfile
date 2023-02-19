FROM rust:1.67 as builder
WORKDIR /usr/src/grache2
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install openssl ca-certificates
RUN update-ca-certificates
COPY --from=builder /usr/local/cargo/bin/grache2 /usr/local/bin/grache2
CMD ["grache2"]