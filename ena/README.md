# ena

Files manager.

## Behavior

The struct "Files" provides access to "FilesEntries" in an unspecified order via the Iterator trait, walking each directory found up to a maximum depth of 5. Symlinks will not be followed.

The "mark" functionality relies on NTFS's Alternate Data Streams feature.

## API

* Files
    * new
    * Iterator trait

* FilesEntry
    * path
    * relative
    * container
    * file_name
    * is_marked
    * toggle_mark
    * move_to_folder
    * delete
    * AsRef trait
