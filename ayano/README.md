# ayano

Non-compliant HTTP/1.1 server.

## Behavior

* `Server`: provides access to `Request` structs via the "accept" method, blocking the execution thread
* `Request`: has a size limit of 64 KiB and each can generate only a single `Response` via the "start_response" method
* `Response`: "Transfer-Encoding: chunked" is always used a will signal EOF on drop

## API

* `Server`
    * bind
    * accept

* `Request`
    * endpoint
    * get_header
    * form_data
    * start_response

* `Response`
    * Write trait
    * Drop trait
