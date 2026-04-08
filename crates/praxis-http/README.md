# praxis-http

[![crates.io](https://img.shields.io/crates/v/praxis-http.svg)](https://crates.io/crates/praxis-http)
[![docs.rs](https://img.shields.io/docsrs/praxis-http)](https://docs.rs/praxis-http)

HTTP protocol rules enforcement via ontology -- connection lifecycle, method semantics, status codes.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models the HTTP protocol where connection state transitions, method semantics, and status codes are enforced by the type system. The connection follows a strict state machine (Idle -> Connecting -> SendingRequest -> AwaitingResponse -> Complete) with retry logic and keep-alive support. Request construction enforces that safe methods cannot carry bodies, and status codes are validated to the 100-599 range.

## Key Types

| Type | Description |
|---|---|
| `Connection` | HTTP connection state machine with retry and keep-alive support |
| `ConnectionState` | Lifecycle phases: Idle, Connecting, SendingRequest, AwaitingResponse, ReceivingResponse, Complete, Error, Closed |
| `ConnectionAction` | State transitions: Connect, SendRequest, ReceiveResponse, Complete, Retry, Close, Reset |
| `Method` | HTTP methods with semantic properties: safety, idempotency, body support |
| `Request` | An HTTP request with method, path, headers, and validated body |
| `Response` | An HTTP response with status code, headers, and body |
| `StatusCode` | Validated HTTP status code (100-599) with class classification |

## Example

```rust
use praxis_http::{Connection, ConnectionAction, Request, Method, StatusCode};

// Connection follows a strict state machine
let conn = Connection::new(3); // max 3 retries
let conn = conn.apply(ConnectionAction::Connect).unwrap();
let conn = conn.apply(ConnectionAction::SendRequest).unwrap();

// Safe methods reject bodies
let get = Request::new(Method::Get, "/api/items");
assert!(get.with_body(vec![1, 2, 3]).is_err());

// POST allows a body
let post = Request::new(Method::Post, "/api/items")
    .with_body(b"hello".to_vec()).unwrap();

// Status codes are validated
assert!(StatusCode::new(999).is_err());
assert!(StatusCode::OK.is_success());
```

## License

CC BY-NC-SA 4.0
