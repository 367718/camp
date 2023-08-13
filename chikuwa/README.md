# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* **EphemeralPath**: Paths that attempt to delete themselves when dropped.
* **WinString**: Null-terminated UTF-16 encoded strings.
* **register_app**: Attempt to prevent more than one instance of an application from running.
* **execute_app**: Run command or open associated external application.
* **current_date**: Current date as string in YYYYMMDD format.
* **percent_encode**: Replace certain characters of a string with a safe representation for inclusion in a URL.
* **natural_cmp**: String comparison that takes numerical values into consideration.
* **insensitive_contains**: Whether supplied elemets are substrings (case-insensitive).
* **tag_range**: Position of elements between two delimiters (case-insensitive).
* **concat_str**: Macro for efficient string concatenation.
