# Installing on Kubernetes with Helm

The chart lives in [`charts/klask`](../../charts/klask) (its
[README](../../charts/klask/README.md) and `values.yaml` document every
parameter). It deploys the frontend, the backend and, optionally, a PostgreSQL
StatefulSet.

## Where the chart is published

The CI publishes the chart as an OCI artifact on the GitHub Container Registry
(workflow `3-publish-helm.yml`); there is no classic Helm repository. Pick a
version from the [releases](https://github.com/klask-dev/klask-dev/releases):

```bash
helm install klask oci://ghcr.io/klask-dev/klask --version <version> \
  --namespace klask --create-namespace -f my-values.yaml
```

You can also install from a checkout: `helm install klask ./charts/klask -f my-values.yaml`.

## Minimal values

Set the secrets explicitly. If they are left empty, the chart generates weaker,
`randAlphaNum`-based values on first install and keeps them across upgrades.

```yaml
backend:
  auth:
    jwtSecret: "<openssl rand -hex 32>"
    encryptionKey: "<openssl rand -hex 16>"   # keep a copy: see Configuration
  # or reference an existing Secret: backend.existingAuthSecret

postgresql:
  enabled: true
  persistence:
    size: 10Gi

ingress:
  enabled: true
  hosts:
    - host: klask.example.com
      paths:
        - path: /
          pathType: Prefix
```

## Exposing Klask

The interface, the REST API (`/api`) and the MCP endpoint (`/mcp`) are served
from one host name. Pick one of:

- **Ingress** (`ingress.enabled`), as in the example above. Works with any
  ingress controller; set `ingress.className`.
- **Gateway API** (`httpRoute.enabled`): the chart creates an `HTTPRoute`
  attached to a Gateway you already run. It does not create the Gateway, its
  listeners or its certificates, and the Gateway must accept routes from the
  namespace of the release (`allowedRoutes`).

  ```yaml
  httpRoute:
    enabled: true
    parentRefs:
      - name: shared-gateway
        namespace: gateway-system
    hostnames:
      - klask.example.com
  ```

- **Istio** (`istio.enabled`): a `Gateway` and a `VirtualService`.
- **None of them**: reach the frontend service (port-forward, NodePort or
  LoadBalancer). Its nginx relays `/api` and `/mcp` to the backend.

In the first three cases `/api` and `/mcp` go straight to the backend service,
and the chart sets `TRUSTED_PROXY_HOPS=1` for the login rate limit; raise
`backend.config.trustedProxyHops` if more proxies sit in front (see
[Configuration](configuration.md#authentication-and-cors)). Set
`backend.config.cookieSecure: true` when the site is served over HTTPS.

With `networkPolicy.enabled` (the default), the pods of the ingress controller
or of the gateway must be allowed in: set
`networkPolicy.ingress.namespaceSelector` to their namespace (default
`ingress-nginx`).

## Secrets and other settings

Secrets can also come from an existing Kubernetes Secret
(`backend.existingAuthSecret`, `backend.existingSecret`,
`postgresql.auth.existingSecret`). Backend settings that are not secret go under
`backend.config` (see `values.yaml`); anything else can be passed as environment
variables through `backend.config.extra` or `backend.env.customVars`. The
variables the backend understands are listed in
[Configuration](configuration.md).

### External database

Disable the bundled PostgreSQL and give the backend a connection string:

```yaml
postgresql:
  enabled: false
backend:
  env:
    database:
      url: "postgresql://user:password@db.example.com:5432/klask"
```

An example is provided in
[`charts/klask/values-external-db.yaml`](../../charts/klask/values-external-db.yaml).

## Storage

- The backend stores the search index on a persistent volume
  (`backend.persistence`, 50Gi by default). While it is enabled the Deployment
  uses the `Recreate` strategy, because a `ReadWriteOnce` volume cannot be
  attached to two pods during a rolling update.
- PostgreSQL uses a StatefulSet volume (`postgresql.persistence`).
- Leave `storageClass` empty to use the cluster default.

## Immutable fields

Kubernetes forbids changing some StatefulSet fields once the object exists. An
upgrade that changes them fails with `updates to statefulset spec for fields
other than ... are forbidden`. For the PostgreSQL StatefulSet this concerns:

- `nameOverride` and `fullnameOverride` (they change the StatefulSet and service
  names);
- `postgresql.persistence.storageClass`;
- `postgresql.persistence.size` can be increased but not reduced.

Image tags, resources, replicas of the backend and frontend, ingress and
probes can be changed freely.

To change an immutable field, back up the data, delete the StatefulSet while
keeping the volume, then upgrade:

```bash
kubectl exec -n klask klask-postgresql-0 -- pg_dump -U klask klask > backup.sql
kubectl delete statefulset klask-postgresql --cascade=orphan -n klask
helm upgrade klask oci://ghcr.io/klask-dev/klask --version <version> -n klask -f my-values.yaml
```

Changing the storage class of an existing volume additionally requires creating
a new volume and restoring the dump into it.

## Upgrade and roll back

```bash
helm diff upgrade ...            # optional, with the helm-diff plugin
helm upgrade klask oci://ghcr.io/klask-dev/klask --version <new-version> -n klask -f my-values.yaml
helm history klask -n klask
helm rollback klask <revision> -n klask
```

[`scripts/safe-helm-deploy.sh`](../../scripts/safe-helm-deploy.sh) wraps an upgrade
with a backup, a dry run and a verification step. Read it before use: it assumes
a kubeconfig at `~/.kube/test` unless `KUBECONFIG` is set. Chart tests that need
no cluster are in [`scripts/test-helm.sh`](../../scripts/test-helm.sh) and
[`Makefile.helm`](../../Makefile.helm) (`make -f Makefile.helm lint`).

## Versions

The chart version follows the application release. For a pull-request build the
CI appends `-pr<number>` or `-dev` to the base version in `Chart.yaml`. Image
tags and the release process are described in [CI/CD](../dev/ci-cd.md).
