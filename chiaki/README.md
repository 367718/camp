# chiaki

Lists manager.

## Behavior

On construction, a file with the specified name and the "ck" extension will be loaded from the current directory.

* The `List` struct provides access to `ListEntry` structs via the "iter" method
* An invalid entry encountered during deserialization or concurrent access to the same list file may lead to data loss
* Symlinks are not supported

## API

* `List`
    * load
    * iter
    * set
    * delete

* `ListEntry`
