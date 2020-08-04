# WPScan-Analyze Dockerfile
FROM rust:latest
RUN mkdir /usr/src/wpscan-analyze/
ADD . /usr/src/wpscan-analyze/
WORKDIR /usr/src/wpscan-analyze
RUN ls -alh
RUN cargo build --verbose
RUN chown -R wpscan-analyze /wpscan-analyze
USER wpscan-analyze
ENTRYPOINT ["wpscan-analyze"]