# chiaki

Lists manager.

## Behavior

The `List` struct provides access to `ListEntry` elements via the "iter" method.

On construction, a file with the specified name and the "ck" extension will be loaded from the current directory. Symlinks are not supported.

An invalid entry encountered while deserialization will cause the next entries, if any, to be skipped. This can lead to data loss.

File size is limited to 512 KiB.

## API

* `List`
    * load
    * iter
    * set
    * delete

* `ListEntry`
