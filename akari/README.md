# akari

HTTP/2 client.

## Behavior

* Based on Microsoft Windows HTTP Services (WinHTTP).
* Only the "GET" request method is supported.
* Connections are pooled for reuse.

## API

* Client
    * new
    * get

* Payload
    * content_length
    * Read trait
