# ena

Files manager.

## Behavior

The `Files` struct provides access to `FilesEntry` structs in an unspecified order via the "Iterator" trait.

The "mark" functionality relies on NTFS's Alternate Data Streams feature.

## API

* `Files`
    * new
    * Iterator trait

* `FilesEntry`
    * path
    * relative
    * container
    * file_name
    * is_marked
    * toggle_mark
    * move_to_folder
    * delete
    * AsRef trait
