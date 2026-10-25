# akari

HTTP/2 client.

## Behavior

The `Client` struct provides access to `Response` structs via the "get" method.

Only HTTPS endpoints are supported and connections will be pooled for reuse.

## API

* `Client`
    * new
    * get

* `Response`
    * Read trait
