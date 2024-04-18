# minami

Files and lists manager.

## Behavior

An HTTP interface allows the listing and manipulation of files, watchlist, rules and feeds.

The application is designed to be run silently in the background.

The main route, "/", presents two lists side by side on screens with at least 1600px of width. Individual lists can be accessed on "/files", "/watchlist", "/rules" and "/feeds".

The following keyboard shortcuts are available:

* **Control + C**: copy the selected entries text to clipboard, inserting linebreaks.
* **Control + X**: copy the selected entries text to clipboard, removing square brackets and parens and inserting linebreaks.

## Configuration parameters used

* **address**: listening address for the web interface.
* **root**: path to files directory.
* **flag**: tag used to mark files as watched.
* **player**: application used to play files.

## List files used

* **watchlist**
* **rules**
* **feeds**
