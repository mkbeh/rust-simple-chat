# Rust HTTP and Postgresql - microservice

![GitHub CI](https://github.com/mkbeh/rust-simple-chat/actions/workflows/ci.yml/badge.svg)

Backend stack using Rust , including interactive API documentation and many useful features out of the box.
Project based on [caslex](https://github.com/mkbeh/caslex).

**Contains:**

* HTTP web server
* Body validation
* JWT
* Middlewares
* API visualizer
* Postgres pool and migrations
* Observability
* Unit tests using  [mockall](https://docs.rs/mockall/latest/mockall/)

### Interactive API documentation

![img](/assets/img/scalar_docs.gif)

## How to use it

### Generate passwords

You will be asked to provide passwords and secret keys for several components. Open another terminal and run:

```
openssl rand -hex 32
# Outputs something like: 99d3b1f01aa639e4a76f4fc281fc834747a543720ba4c8a8648ba755aef9be7f
```

## How to deploy

```bash
docker-compose up --build -d
```

### Scalar UI

http://localhost:9000/docs

### Prometheus

http://localhost:9007/metrics

### Jaeger UI

http://localhost:16686/search