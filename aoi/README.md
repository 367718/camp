# aoi

Remote control of the mpv player.

## Behavior

The frontend is an HTTP interface, while the backend is a named pipe.

## Issues and limitations

* Connections are not kept alive.
* The index data is sent on every response, which might be wasteful.
* No timeout mechanism has been implemented for the write operations on the named pipe.
