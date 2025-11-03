# minami

Files and lists manager.

## Behavior

An HTTP interface allows the listing and manipulation of files, watchlist, rules and feeds.

The following keyboard shortcuts are available:

* `Control + C`: copy selected entries to clipboard, inserting linebreaks.
* `Control + X`: copy selected entries to clipboard, performing a cleanup and inserting linebreaks.

## Routes available

* `/`: two lists side by side on screens with at least 1600px of width.
* `/files`: entries present in the configured directory.
* `/watchlist`: entries present in the corresponding list file.
* `/rules`: entries present in the corresponding list file.
* `/feeds`: entries present in the corresponding list file.
* `/mobile`: entries present in the configured directory, intented to be used by tablet devices.

## Configuration parameters used

* `address`: listening address for the web interface.
* `root`: path to files directory.
* `flag`: tag used to mark files.
* `player`: application used to play files.

## List files used

* `watchlist`
* `rules`
* `feeds`
