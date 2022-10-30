# aoi

Remote control of the mpv player.

## Behavior

The frontend is an HTTP interface, while the backend is a named pipe.

## Issues and limitations

* Provided bind address must be IPv4.
* Connections are not kept alive for later reuse.
* The index data is sent on every response, which might be wasteful.
* No timeout mechanism has been implemented for the write operations on the named pipe.
