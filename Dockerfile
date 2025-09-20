FROM rust:latest as build-stage
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path backend
RUN cargo install --path client_logger
RUN cargo install --path notifier

FROM rust:slim
COPY --from=build-stage /usr/local/cargo/bin/backend /usr/local/bin/backend
COPY --from=build-stage /usr/local/cargo/bin/client_logger /usr/local/bin/client_logger
COPY --from=build-stage /usr/local/cargo/bin/notifier /usr/local/bin/notifier
CMD ["backend"]
