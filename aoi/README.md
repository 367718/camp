# aoi

mpv player remote controller.

## Behavior

The frontend is an HTTP interface, while the backend is a named pipe.

Configuration parameters used:

* **name**: path to the named pipe used by mpv.
* **bind**: listening address for the web interface.

## Issues and limitations

* Only IPv4 addresses are supported.
* Connections are not kept alive for later reuse.
* No timeout mechanism has been implemented for the write operations on the named pipe.
