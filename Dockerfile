FROM rust:1.75.0
WORKDIR /usr/src/wictk
ARG APIKEY
ENV OPENWEATHERMAPAPIKEY=$APIKEY
COPY . .
RUN cargo install --path .
CMD ["wictk"]
