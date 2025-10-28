# chiaki

Lists manager.

## Behavior

The `List` struct provides access to `ListEntry` elements via the "iter" method.

On construction, a file with the specified name and the "ck" extension will be loaded from the current directory. Symlinks are not supported.

The maximum allowed size of the loaded list is 512 KiB.

An invalid entry encountered while loading a list will cause the next entries, if any, to be skipped.

## API

* `List`
    * load
    * iter
    * set
    * delete

* `ListEntry`
