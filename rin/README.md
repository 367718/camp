# rin

Configuration files manager.

## Behavior

A file with the same name as the application but with extension "rn" is expected to be found alongside the executable, and will be loaded on the first attempted access to a parameter. A failure to find or read the file will trigger a panic.

Keys and values are separated by " = " (e.g. "path = \example\").

Lines must end with "\r\n".
