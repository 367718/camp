# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* **EphemeralPath**: Paths that attempt to delete themselves when dropped.
* **WinString**: Null-terminated UTF-16 encoded strings.
* **open_resource**: Open provided resource with external application.
* **percent_encode**: Replace certain characters of a string with a safe representation for inclusion in a URL.
* **tag_range**: Position of elements between two delimiters (case-insensitive).
* **concat_str**: Macro for efficient string concatenation.

## Dependencies

The "open_resource" functionality requires the linking of the "shell32" windows lib.
