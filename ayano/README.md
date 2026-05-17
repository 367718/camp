# ayano

Non-compliant HTTP/1.1 server.

## Behavior

The `Server` struct provides access to `Request` structs via the "Iterator" trait, blocking the execution thread. A single `Response` struct may be generated from each `Request`.

* `Server`: "Connection: Keep-Alive" is not supported
* `Request`: has a size limit of 64 KiB
* `Response`: "Transfer-Encoding: chunked" is always used

## API

* `Server`
    * bind
    * accept

* `Request`
    * endpoint
    * get_header
    * body_len
    * query_string
    * form_params
    * start_response

* `Response`
    * Write trait
    * Drop trait
