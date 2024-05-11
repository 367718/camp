# ayano

Non-compliant HTTP/1.1 server.

## Behavior

The `Server` struct provides access to `Request` elements via the "Iterator" trait.

On construction, the specified address will be bound, blocking the execution thread.

Each `Request` has a size limit of 512 KiB, and can be used to spawn one and only one `Response`.

The timeouts for send and recieve are each set at 5 seconds.

## API

* `Server`
    * bind
    * Iterator trait

* `Request`
    * resource
    * param
    * start_response

* `Response`
    * Write trait
    * Drop trait
