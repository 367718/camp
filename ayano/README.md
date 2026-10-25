# ayano

HTTP/1.1 server.

## Behavior

* `Server`
    * provides access to `Request` structs via the "accept" method, blocking the execution thread

* `Request`:
    * accepts HTTP/1.1 only
    * limits headers size to 16 KiB
    * limits body size to 512 KiB
    * generates only a single `Response` each via the "start_response" method

* `Response`: 
    * "Transfer-Encoding: chunked" is always used and will signal EOF on drop

## API

* `Server`
    * bind
    * accept

* `Request`
    * verb
    * path
    * form_field
    * start_response

* `Response`
    * Write trait
    * Drop trait
