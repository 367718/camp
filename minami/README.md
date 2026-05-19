# minami

Files and lists manager.

## Behavior

An HTTP interface allows the access and manipulation of files and lists entries.

## Routes available

* `/`: two lists side by side on screens with at least 1600px of width
* `/files`: entries present in the configured directory
* `/watchlist`: entries present in the corresponding list file
* `/rules`: entries present in the corresponding list file
* `/feeds`: entries present in the corresponding list file

## Leyboard shortcuts available

* `Control + C`: copy selected entries to clipboard, inserting linebreaks
* `Control + X`: copy selected entries to clipboard, performing a cleanup and inserting linebreaks

## Configuration parameters used

* `address`: listening address for the web interface
* `root`: path to files directory
* `flag`: tag used to mark files
* `player`: application used to play files
* `max_directory_depth`: maximum allowed traversal depth for files list, starting from 1
* `max_list_size`: maximum allowed file size for watchlist, rules and feeds lists, in bytes

## List files used

* `watchlist`
* `rules`
* `feeds`
