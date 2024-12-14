# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: paths that attempt to delete themselves when dropped.
* `escape_html`: escape bytes for safe usage in an HTML context.
* `win_string`: UTF-16 encoded and null-terminated string.
* `subslice_range`: position of subslice between two delimiters (case-insensitive).

## API

* `EphemeralPath`
    * make_permanent
    * From trait
    * Deref trait
    * AsRef trait
    * Drop trait

* `escape_html`

* `win_string`

* `subslice_range`
