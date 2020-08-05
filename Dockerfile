# WPScan-Analyze Dockerfile
FROM rust:latest
RUN mkdir /wpscan-analyze/
ADD . /wpscan-analyze/
WORKDIR /wpscan-analyze
RUN ls -alh
RUN cargo install --path .
ENTRYPOINT ["wpscan-analyze"]