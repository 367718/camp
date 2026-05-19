# akari

HTTP/2 client.

## Behavior

The `Client` struct provides access to `Response` structs via the "get" method.

Only HTTPS is supported and connections will be pooled for reuse.

## API

* `Client`
    * new
    * get

* `Response`
    * content_length
    * Read trait
