# ena

Files manager.

## Behavior

The `Files` struct provides access to `FilesEntry` elements in an unspecified order via the "Iterator" trait.

On construction, the specified directory will be walked up to a maximum depth of 5. Symlinks will not be followed.

The "mark" functionality relies on NTFS's Alternate Data Streams feature.

## API

* `Files`
    * walk
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
