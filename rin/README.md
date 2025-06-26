# rin

Configuration parameters provider.

## Behavior

On the first access to a parameter, a file with the same name as the executable but with the "rn" extension will be loaded from the current working directory. The data will be kept in memory.

Keys and values must be separated by " = " (e.g. "path = \example\"), while lines must end with "\r\n".

A failure during load will cause a panic.

File size is limited to 32 KiB.

## API

* `get`
