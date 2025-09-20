# ayano

Non-compliant HTTP/1.1 server.

## Behavior

The `Server` struct provides access to `Request` elements via the "Iterator" trait, blocking the execution thread.

`Server`: "Connection: Keep-Alive" is not supported and will be ignored
`Request`: has a size limit of 512 KiB and neither "Transfer-Encoding" nor "Content-Type: application/x-www-form-urlencoded" are supported
`Response`: only one per `Request` may be started and "Transfer-Encoding: chunked" is always used

## API

* `Server`
    * bind
    * Iterator trait

* `Request`
    * method_and_path
    * get_header
    * body_len
    * query_string
    * form_data
    * start_response

* `Response`
    * Write trait
    * Drop trait
