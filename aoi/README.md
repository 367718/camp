# aoi

mpv player remote controller.

## Behavior

An HTTP interface allows the remote control of a running mpv instance. The parameter "input-ipc-server" of mpv must be configured with the name of the pipe to use for communication.

No timeout mechanism has been implemented for the write operations on the named pipe.

Only one route is available, "/".

## Configuration parameters used

* **address**: listening address for the web interface.
* **name**: path to the named pipe used by mpv.
