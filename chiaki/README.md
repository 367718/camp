# chiaki

Lists manager.

## Behavior

* List files must be created manually and have the "ck" extension.
* Only files located alongside the executable will be loaded.
* Symlinked files are not supported.
* Multiple concurrent accesses to the same file are discouraged.
* To prevent data loss, loading of lists that contains a tag too large for the target platform will be aborted.
* UTF-8 correctness is not enforced.
* Stored tags are required to be unique.
