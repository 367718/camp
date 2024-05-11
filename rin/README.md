# rin

Configuration parameters provider.

## Behavior

On the first attempted access to a parameter, a file with the same name as the executable but with the "rn" extension will be loaded. Data will be kept immutable.

Keys and values must be separated by " = " (e.g. "path = \example\"), while lines must end with "\r\n".

A failure during load will cause a panic.

## API

* `get`
