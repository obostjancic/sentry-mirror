# Sentry Mirror

:warning: This application is not ready for customer use yet.

This application helps customers create data-continuity for ingest traffic
during a region relocation, or self-hosted to saas relocation. This application
will accept inbound ingest traffic on a configured DSN and forward events to one
or more outbound DSNs.

Events will be mirrored in a *best-effort* fashion. Delivery to outbound DSNs
will not be buffered, and events in each of the destination organizations may be
sampled differently.

## Configuration

sentry-mirror is primary configured through a YAML file:

```yaml
port: 3000
keys:
  - inbound: http://public-key@sentry-mirror.acme.org/1847101
    outbound:
      - https://public-key-red@o123.ingest.de.sentry.io/123456
      - https://public-key-blue@o456.ingest.us.sentry.io/654321
```

## Request rewriting

When events are mirrored to outbound DSNs the following modifications may be made the received requests:

1. `sentry_key` component of `Authorization` and `X-Sentry-Auth` headers will be replaced.
2. `dsn` in envelope headers will be replaced.
3. `trace.public_key` in envelope headers will be replaced.
4. Content-Length, Content-Encoding, Host, X-Forwarded-For headers will be removed.

## Deployment

[sentry mirror](sentry-mirror) is packaged as a Docker container that can be deployed and operated in customer environments. sentry-mirror needs to have SSL terminated externally and should be put behind a load-balancer or reverse proxy.

### Build

```shell
# Build the image
docker build -f Dockerfile -t sentry-mirror .
```

### Run

```
# Mount your configuration file into the container and run the application
docker run --name sentry-mirror -v ./config.yml:/opt/config.yml sentry-mirror /opt/sentry-mirror -c /opt/config.yml
```

If you map the application to a port that isn't 3000 you'll also need to expose the port in the container.
sentry-mirror will need to be operated behind a load balancer as it cannot terminate SSL connections

