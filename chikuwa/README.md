# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: paths that attempt to delete themselves when dropped.
* `WinString`: null-terminated UTF-16 encoded strings.
* `HtmlEscaper`: escape bytes for safe usage in an HTML context.
* `RssFeed`: entries of an RSS feed, with mandatory "title" and "link" fields.
* `subslice_range`: position of subslice between two delimiters (case-insensitive).
* `first_number`: first ocurrence of a number in an slice of bytes, interpreted as an unsigned integer.
* `write_to_named_pipe`: attempt to write supplied data to specified win32 named pipe.

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

* `HtmlEscaper`
    * From trait
    * Iterator trait

* `RssFeed`
    * new
    * Iterator trait

* `subslice_range`

* `first_number`

* `write_to_named_pipe`
