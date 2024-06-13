# aoi

mpv player remote controller.

## Behavior

An HTTP interface allows the remote control of a running mpv instance.

The parameter "input-ipc-server" of mpv must be configured with the name of the pipe to use for communication.

## Routes available

* `/`: all the controls available, intended to be used by a tablet device.

## Configuration parameters used

* `address`: listening address for the web interface.
* `pipe`: path to the named pipe used by mpv.
