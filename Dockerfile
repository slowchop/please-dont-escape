FROM rust:latest
COPY . /build
WORKDIR /build
RUN make web