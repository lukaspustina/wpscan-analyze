# WPScan-Analyze Dockerfile


FROM rust:latest
COPY ./* /usr/src/wpscan-analyze/
WORKDIR /usr/src/wpscan-analyze
RUN ls -alh
RUN cargo build --verbose
RUN chown -R wpscan-analyze /wpscan-analyze
USER wpscan-analyze
ENTRYPOINT ["wpscan-analyze"]