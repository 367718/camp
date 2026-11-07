# akari

HTTP/2 client.

## Behavior

* The `Client` struct provides access to `Response` structs via the "get" method
* Only HTTPS connections are supported
* Connections will be pooled for reuse
* Redirects will not be followed automatically
* Requires Windows 10 version 1607 or later

## API

* `Client`
    * new
    * get

* `Response`
    * status_code
    * Read trait
