# akari

HTTP/2 client.

## Behavior

The `Client` struct provides access to `Payload` elements via the "get" method.

Connections will be pooled for reuse.

The timeouts for resolution, connection, send and recieve are each set at 15 seconds.

## API

* `Client`
    * new
    * get

* `Payload`
    * content_length
    * Read trait
