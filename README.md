# Sentry Mirror

:warning: This application is not ready for customer use yet.

This application helps customers create data-continuity for ingest traffic
during a region relocation, or self-hosted to saas relocation. This application
will accept inbound ingest traffic on a configured DSN and forward events to one
or more outbound DSNs.

Events will be mirrored in a *best-effort* fashion. Delivery to outbound DSNs will
not be buffered, and events in the destination organizations may be sampled differently
by the receiving organizations.

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

## Deployment

sentry-mirror is packaged as a Docker container that can be deployed and operated in customer environments. sentry-mirror needs to have SSL terminated externally and should be put behind a load-balancer or reverse proxy.

