FROM rust:1.72 
WORKDIR /usr/src/wictk
COPY . .
ARG APIKEY
ENV OPENWEATHERMAPAPIKEY=$APIKEY
RUN cargo install --path .
CMD ["wictk"]
