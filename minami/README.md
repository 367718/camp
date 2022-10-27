# minami

Manage files and watchlists.

## Behavior

If the "dlonly" command-line flag is specified, only the "Download new releases" procedure will be run.

A stylesheet file can be used to customize the appearance via a subset of CSS. An example is provided in the "rsc" directory. The file, without changing its name, should be placed alongside the executable, or a path should be provided via the "--stylesheet" argument.

## Issues and limitations

* Only one instance of the application can be running at one time.
* Certain actions, such as related candidates removal on parent series edit, are not rolled back on later failure.
* Global search may not work with pasted text if a character has not been manually entered or deleted at least once.

## Dependencies

* mingw-w64-x86_64-gcc
* mingw-w64-x86_64-pkgconf
* mingw-w64-x86_64-gtk3
