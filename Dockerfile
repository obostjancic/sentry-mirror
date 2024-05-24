# Build image
FROM rust:1.77-bullseye as build

COPY ./ /opt/src

RUN cd /opt/src \
 && cargo build --release

# Runtime image
FROM debian:bullseye

EXPOSE 3000

COPY --from=build /opt/src/target/release/sentry-mirror /opt/sentry-mirror
CMD ["/opt/sentry-mirror"]
