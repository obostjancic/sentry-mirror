# Build image
FROM rust:1.77-bullseye as build

COPY ./ /opt/src

RUN cd /opt/src \
 && cargo build --release

# Runtime image
FROM debian:bullseye

RUN apt-get update && \
  apt-get install -y ca-certificates

EXPOSE 8080

COPY --from=build /opt/src/target/release/sentry-mirror /opt/sentry-mirror
CMD ["/opt/sentry-mirror"]
