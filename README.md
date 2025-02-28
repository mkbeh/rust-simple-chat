# Rust Axum and Postgresql - microservice

![Platform](https://img.shields.io/badge/platform-linux-green.svg)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/Naereen/StrapDown.js/blob/master/LICENSE)
![GitHub CI](https://github.com/mkbeh/rust-simple-chat/actions/workflows/ci.yml/badge.svg)

Backend stack using Rust , including interactive API documentation and many useful features out of the box.

**Full list what has been used:**

* [axum](https://docs.rs/axum/latest/axum/) - web application framework
* [clap](https://docs.rs/clap/latest/clap/) - command line argument parser
* [tokio-postgres](https://docs.rs/tokio-postgres/latest/tokio_postgres/) - an asynchronous, pipelined, PostgreSQL
  client
* [deadpool-postgres](https://docs.rs/deadpool-postgres/latest/deadpool_postgres/) - dead simple async pool for
  connections and objects of any type
* [validator](https://docs.rs/validator/latest/validator/) - struct validator
* [jsonwebtoken](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/) - json web token
* [tracing](https://docs.rs/tracing/latest/tracing/) - a scoped, structured logging and diagnostics system
* [utoipa](https://docs.rs/utoipa/latest/utoipa/) - provides auto-generated OpenAPI documentation for Rust REST APIs

### Interactive API documentation

![cut_output.gif](/assets/img/scalar.gif)

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

### Swagger UI

http://localhost:9000/swagger-ui

### Redoc UI

http://localhost:9000/redoc

### Scalar UI

http://localhost:9000/scalar

### Rapidoc UI

http://localhost:9000/rapidoc