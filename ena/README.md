# ena

Files manager.

## Behavior

The `Files` struct provides access to `FilesEntry` structs in an unspecified order via the "Iterator" trait.

The "mark" functionality relies on the Alternate Data Streams (ADS) feature of the NT File System.

## API

* `Files`
    * new
    * Iterator trait

* `FilesEntry`
    * path
    * relative
    * is_marked
    * toggle_mark
    * move_to_folder
    * delete
    * AsRef trait
