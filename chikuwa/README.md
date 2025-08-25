# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: generate paths that attempt to delete themselves when dropped.
* `escape_html`: escape bytes for safe usage in an HTML context.
* `first_number`: extract the first number found as a positive integer.
* `subslice_range`: find position of subslice between two delimiters (case-insensitive).
* `win_filename`: sanitize string for use as filename in Windows.
* `win_string`: generate UTF-16 null-terminated string.

## API

* `EphemeralPath`
    * make_permanent
    * From trait
    * Deref trait
    * AsRef trait
    * Drop trait

* `escape_html`

* `subslice_range`

* `win_filename`

* `win_string`
