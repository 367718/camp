# rin

Configuration files manager.

## Behavior

* Files must be created and modified manually and have the same name as the executable but with the "rn" extension.
* Only a file located alongside the executable will be loaded.
* File loading will be triggered on the first attempted access to a parameter.
* A failure during file loading will result in a panic.
* Keys and values are separated by " = " (e.g. "path = \example\").
* Lines must end with "\r\n".
