# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: paths that attempt to delete themselves when dropped.
* `WinString`: null-terminated UTF-16 encoded strings.
* `subslice_range`: position of subslice between two delimiters (case-insensitive).
* `HtmlEscaper`: escape bytes for safe usage in an HTML context.

## API

* `EphemeralPath`
    * make_permanent
    * From trait
    * Deref trait
    * AsRef trait
    * Drop trait

* `WinString`
    * From trait
    * Deref trait

* `subslice_range`

* `HtmlEscaper`
    * From trait
    * Iterator trait
