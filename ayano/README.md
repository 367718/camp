# ayano

Non-compliant HTTP/1.1 server.

## Behavior

* `Server`: provides access to `ServerRequest` structs via the "accept" method, blocking the execution thread
* `ServerRequest`: has a size limit of 64 KiB and each can generate only a single `ServerResponse` via the "start_response" method
* `ServerResponse`: "Transfer-Encoding: chunked" is always used a will signal EOF on drop

## API

* `Server`
    * bind
    * accept

* `ServerRequest`
    * endpoint
    * form_data
    * start_response

* `ServerResponse`
    * Write trait
    * Drop trait
