# ena

Files manager.

## Behavior

* From the provided directory, the maximum allowed depth is 5.
* Entries whose paths contains characters not representable in UTF-8 will be skipped.
* The "mark" functionality relies on NTFS's Alternate Data Streams feature.
