# CORS Proxy

A simple proxy that adds all possible CORS headers to proxied responses.

Currently trues to use TLS on first connection and falls back to plain HTTP.

## Usage

Running

```bash
cargo run -- --port 8000 --proxy-to localhost:3000
```

should create a proxy server on port 8000 that proxies all requests to localhost:3000 and adds CORS headers to responses.

```bash
$ curl -sv localhost:8000
...
> GET / HTTP/1.1
> Host: localhost:8000
> User-Agent: curl/8.7.1
> Accept: */*
>
* Request completely sent off
< HTTP/1.1 200 OK
< vary: Origin
< access-control-expose-headers: location, permissions-policy, referrer-policy, vary, x-clacks-overhead, x-content-type-options, date, content-length, content-type
< access-control-allow-origin: *
< access-control-allow-headers: *
< access-control-allow-methods: *
< X-CorsProxy-Request-Id: 018ee6306fbb797f850d789f97c326bb
...
```

and you should see something like the following in your proxy logs

```log
2024-04-16T09:15:17.819462Z  INFO req{t=127.0.0.1:3000 id=018ee6306fbb797f850d789f97c326bb m=GET p=/}: cors_proxy::services::add_cors_headers: Incoming request
2024-04-16T09:15:17.822085Z  INFO req{t=127.0.0.1:3000 id=018ee6306fbb797f850d789f97c326bb m=GET p=/ dur=1.655978ms}: cors_proxy::services::add_cors_headers: Done
```

If you send a request with an `Origin` header, you get a specific CORS response:

```bash
$ curl -sv --header 'Origin: allypost.net' localhost:8000
...
> GET / HTTP/1.1
> Host: localhost:8000
> User-Agent: curl/8.7.1
> Accept: */*
>
* Request completely sent off
< HTTP/1.1 200 OK
< vary: Origin
< access-control-expose-headers: server, date, connection, content-type, content-length
< access-control-allow-origin: allypost.net
< access-control-allow-credentials: true
< X-CorsProxy-Request-Id: 018ee9b26be67aa48751cd8d3569ac15
...
```

## Building

To build the project, run

```bash
cargo build --release
```

and you should get a `cors-proxy` binary in your `target/release` directory.

### Docker

Alternatively, you can build the Docker image with the provided `Dockerfile`/`compose.yaml` configurations.

```bash
docker build --pull --compress \
    --tag 'cors-proxy:latest' \
    --file './.docker/app/Dockerfile' .
```
