# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: generate paths that attempt to delete themselves when dropped.
* `escape_html`: escape bytes for safe usage in an HTML context.
* `first_number`: extract the first number found as a positive integer.
* `insensitive_contains`: check needle presence in haystack (case-insensitively).
* `percent_decode`: decode percent-encoded bytes.
* `split_on_separator`: split on provided separator, returning unmodified input if not found.
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

* `first_number`

* `insensitive_contains`

* `percent_decode`

* `split_on_separator`

* `subslice_range`

* `win_filename`

* `win_string`
