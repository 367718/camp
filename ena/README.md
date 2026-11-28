# ena

Files manager.

## Behavior

* The `Files` struct provides access to `FilesEntry` structs in an unspecified order via the "Iterator" trait
* Paths are limited to MAX_PATH (260 characters)
* Extended-length paths (\\?\) are not supported by "trash"

## API

* `Files`
    * new
    * Iterator trait

* `FilesEntry`
    * relative
    * to_path
    * move_to_folder
    * trash
