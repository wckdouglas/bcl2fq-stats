FROM --platform=linux/x86_64/v8 rust:1.65.0-slim-buster as builder
RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/* 

FROM builder as build
COPY . /opt/
WORKDIR /opt/
RUN cargo install --path .

FROM --platform=linux/x86_64/v8 debian:buster-slim as exec
RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/* 
COPY --from=build /usr/local/cargo/bin/bcl2fq-stats /usr/local/bin/bcl2fq-stats
RUN /usr/local/bin/bcl2fq-stats -h
ENTRYPOINT ["/usr/local/bin/bcl2fq-stats"]
