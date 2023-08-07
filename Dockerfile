FROM rust:1.67 as builder
WORKDIR /usr/src/wictk
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/wictk /usr/local/bin/wictk
CMD ["wictk"]
