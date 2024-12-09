# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* `EphemeralPath`: paths that attempt to delete themselves when dropped.
* `HtmlEscaper`: escape bytes for safe usage in an HTML context.
* `RssFeed`: entries of an RSS feed, with mandatory "title" and "link" fields.
* `win_string`: UTF-16 encoded and null-terminated string.
* `subslice_range`: position of subslice between two delimiters (case-insensitive).
* `first_number`: first ocurrence of a number in an slice of bytes, interpreted as an unsigned integer.

## API

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

* `win_string`

* `subslice_range`

* `first_number`
