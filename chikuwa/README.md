# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: paths that attempt to delete themselves when dropped.
* `WinString`: null-terminated UTF-16 encoded strings.
* `NamedPipe`: access to win32 named pipes, with support for write operations.
* `HtmlEscaper`: escape bytes for safe usage in an HTML context.
* `RssFeed`: entries of an RSS feed, with mandatory "title" and "link" fields.
* `subslice_range`: position of subslice between two delimiters (case-insensitive).
* `first_number`: first ocurrence of a number in an slice of bytes, interpreted as an unsigned integer.

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

* `NamedPipe`
    * connect
    * Write trait

* `HtmlEscaper`
    * From trait
    * Iterator trait

* `RssFeed`
    * new
    * Iterator trait

* `subslice_range`

* `first_number`
