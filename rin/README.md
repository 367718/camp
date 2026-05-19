# rin

Configuration parameters provider.

## Behavior

On first access to any parameter, a file with the executable name and the "rn" extension will be loaded from the current directory.

* Keys and values must be separated by "=", while lines must end with "\r\n"
* Only values of type "&str" and "u64" are supported
* A failure during load will cause a panic
* Symlinks are not supported
* File size is limited to 32 KiB

## API

* `get`
