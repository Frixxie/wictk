FROM rust:latest as build-stage
WORKDIR /usr/src/app
ARG APIKEY
ENV OPENWEATHERMAPAPIKEY=$APIKEY
COPY . .
RUN cargo install --path .

FROM rust:slim
COPY --from=build-stage /usr/local/cargo/bin/wictk /usr/local/bin/wictk
CMD ["wictk"]
