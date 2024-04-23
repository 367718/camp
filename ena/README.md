# ena

Files manager.

## Behavior

* From the provided directory, the maximum allowed depth is 5.
* The "mark" functionality relies on NTFS's Alternate Data Streams feature.

## API

* Files
    * new
    * Iterator trait

* FilesEntry
    * relative
    * components
    * is_marked
    * toggle_mark
    * move_to_folder
    * delete
    * AsRef trait
