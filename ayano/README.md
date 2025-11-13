# ayano

Non-compliant HTTP/1.1 server.

## Behavior

The `Server` struct provides access to `Request` elements via the "Iterator" trait, blocking the execution thread.

* `Server`: "Connection: Keep-Alive" is not supported and will be ignored
* `Request`: has a size limit of 512 KiB and "Transfer-Encoding" is not supported
* `Response`: only one per `Request` may be started and "Transfer-Encoding: chunked" is always used

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
