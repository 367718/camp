# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `delimited_range`: find range between two delimiters (case-insensitive).
* `EphemeralPath`: generate paths that attempt to delete themselves when dropped.
* `escape_html`: escape bytes for safe usage in an HTML context.
* `first_number`: extract the first number found as a positive integer.
* `percent_decode`: decode percent-encoded bytes.
* `split_once`: split on provided separator (case-insensitive), returning unmodified input if not found.
* `subslice_index`: find position of subslice (case-insensitive).
* `win_filename`: sanitize string for use as filename in Windows.
* `win_string`: generate UTF-16 null-terminated string.

## API

* `delimited_range`

* `EphemeralPath`
    * make_permanent
    * From trait
    * Deref trait
    * AsRef trait
    * Drop trait

* `escape_html`

* `first_number`

* `percent_decode`

* `split_once`

* `subslice_index`

* `win_filename`

* `win_string`
