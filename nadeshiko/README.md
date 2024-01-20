# nadeshiko

Torrent files downloader.

## Behavior

If provided an url to an RSS feed, any entry considered relevant will be downloaded, saving the content available in the address specified by the "link" field to a torrent file.

An entry is considered relevant if:

- A rule tag is defined such that the start of the "title" field can be case-sensitively matched
- An episode number can be extracted from the "title" field without considering the matched rule tag portion
- The extracted episode number is greater than the matched rule value

Since any problem will cause the application to terminate early and the process consists of first creating the torrent file and then attempting to update the rules list, any scenario in which the list operation is not completed successfully can cause problems: a re-run of the application might attempt to download a torrent file that already exists in the disk drive, generating a new error.

## Configuration and lists

Configuration parameters used:

* **folder**: destination for the downloaded torrents.

List files used:

* **rules**
* **feeds**
