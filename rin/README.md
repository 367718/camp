# rin

Configuration parameters provider.

## Behavior

On first access to any parameter, a file with the executable name and the "rn" extension will be loaded from the current directory. Symlinks are not supported. The data will be kept in memory.

Keys and values must be separated by "=", while lines must end with "\r\n".

A failure during load will cause a panic.

File size is limited to 32 KiB.

## API

* `get`
