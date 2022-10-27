# ena

File operations.

## Behavior

The "add" and "remove" methods accept both a file path and a directory path.

The "mark" functionality is implemented using NTFS Alternate Data Streams.

The "perform_maintenance" method will delete every file marked as "updated" or considered irrelevant and every directory considered empty.

The "refresh_queue" method is used to keep a sublist of files. Entries will preserve the order in which they were originally provided, and will be discarded if not present in subsequent refreshes. Containers are supported, and will take precedence over its contents.

## Issues and limitations

* No distinction on initialization between an error reading the provided path and an absence of relevant entries.
* A type must be specified if no folder name is supplied when invoking "move_to_folder".
* Symbolic links and junction points are not allowed.
