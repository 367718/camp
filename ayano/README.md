# ayano

Non-compliant HTTP/1.1 server.

## Behavior

* "Connection: Keep-Alive" is not supported and will be ignored, alongside most other headers.
* Only one Response per Request can be sent.
* The "Transfer-Encoding" used for the Response is always "chunked".
* EOF will be signaled on Response drop.
* Both the read and write timeouts are set to 5 seconds.
* Request size is limited to 512 KB.

## API

* Server
    * new
    * Iterator trait

* Request
    * resource
    * param
    * start_response

* Response
    * Write trait
    * Drop trait
