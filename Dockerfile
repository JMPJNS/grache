FROM rust:1.67-alpine as builder
RUN apk add openssl openssl-dev libc-dev
WORKDIR /usr/src/grache2
COPY . .
RUN cargo install --path .

FROM alpine
RUN apk add openssl
COPY --from=builder /usr/local/cargo/bin/grache2 /usr/local/bin/grache2
CMD ["grache2"]