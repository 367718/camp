# chiaki

Lists manager.

## Behavior

A key-value store, where keys are byte slices and values are 64-bit unsigned integers.

The `List` struct provides access to `ListEntry` elements via the "iter" method.

On construction, a file with the specified name and the "ck" extension will be loaded from the current working directory. Symlinks are not supported.

## API

* `List`
    * load
    * iter
    * insert
    * update
    * delete

* `ListEntry`
