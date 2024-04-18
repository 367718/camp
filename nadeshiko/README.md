# nadeshiko

Torrent files downloader.

## Behavior

From an RSS feed URL, any entry considered relevant will be downloaded, saving the content available in the address specified by the "link" field to a torrent file.

An entry is considered relevant if:

* A rule tag is defined such that the start of the "title" field can be case-sensitively matched.
* An episode number can be extracted from the "title" field without considering the matched rule tag portion.
* The extracted episode number is greater than the matched rule value.

## Configuration parameters used

* **folder**: destination for the downloaded torrents.

## List files used

* **rules**
* **feeds**
