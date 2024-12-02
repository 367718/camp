# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `win_str`: macro for creating UTF-16 encoded and null-terminated strings.
* `EphemeralPath`: paths that attempt to delete themselves when dropped.
* `HtmlEscaper`: escape bytes for safe usage in an HTML context.
* `RssFeed`: entries of an RSS feed, with mandatory "title" and "link" fields.
* `subslice_range`: position of subslice between two delimiters (case-insensitive).
* `first_number`: first ocurrence of a number in an slice of bytes, interpreted as an unsigned integer.
* `write_to_named_pipe`: attempt to write supplied data to specified win32 named pipe.

## API

* `win_str!`

* `EphemeralPath`
    * make_permanent
    * From trait
    * Deref trait
    * AsRef trait
    * Drop trait

* `HtmlEscaper`
    * From trait
    * Iterator trait

* `RssFeed`
    * new
    * Iterator trait

* `subslice_range`

* `first_number`

* `write_to_named_pipe`
