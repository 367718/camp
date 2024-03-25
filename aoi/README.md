# aoi

mpv player remote controller.

## Behavior

* The frontend is an HTTP interface, while the backend is a named pipe.
* No timeout mechanism has been implemented for the write operations on the named pipe.

## Configuration parameters used

* **address**: listening address for the web interface.
* **name**: path to the named pipe used by mpv.
