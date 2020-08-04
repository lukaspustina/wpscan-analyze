# WPScan-Analyze Dockerfile
FROM rust:latest
RUN mkdir /usr/src/wpscan-analyze/
ADD . /usr/src/wpscan-analyze/
WORKDIR /usr/src/wpscan-analyze
RUN ls -alh
RUN cargo build
RUN cargo install --path .
ENTRYPOINT ["wpscan-analyze"]