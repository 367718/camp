# chiaki

Lists manager.

## Behavior

The `List` struct provides access to `ListEntry` elements via the "iter" method.

On construction, a file with the specified name and the "ck" extension will be loaded from the directory where the executable resides. Symlinks will not be followed.

## API

* `List`
    * load
    * iter
    * insert
    * update
    * delete

* `ListEntry`
