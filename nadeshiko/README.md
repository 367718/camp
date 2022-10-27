# nadeshiko

File downloads and series updates.

## Behavior

This library only determines downloads and updates opportunities. It does not perform them.

The download functionality expects an XML conforming to the RSS 2.0 spec whose channel items contain both a "title" and "link" tags.

## Issues and limitations

* Chapter numbers must be higher than zero.
* If a chapter number is determined to have a decimal point, such as "10.5", the entry will be skipped.
* Chapter number extraction will be unreliable if candidate cleanup is not done properly.
* Entries whose fields contain characters not representable in UTF-8 will be skipped.
