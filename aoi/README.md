# aoi

Remote control of the mpv player.

## Behavior

The frontend is an HTTP interface, while the backend is a named pipe.

## Issues and limitations

* Only IPv4 addresses are supported.
* Connections are not kept alive for later reuse.
* No timeout mechanism has been implemented for the write operations on the named pipe.
