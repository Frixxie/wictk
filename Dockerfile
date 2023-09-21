FROM rust:1.72.1 
WORKDIR /usr/src/wictk
ARG APIKEY
ENV OPENWEATHERMAPAPIKEY=$APIKEY
COPY . .
RUN cargo install --path .
CMD ["wictk"]
