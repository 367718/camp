# minami

Manage files and watchlists.

## Behavior

Use the "--help" command-line flag to list all available flags and arguments.

The application appearance can be customized via a subset of CSS. An example of the rules applied by default can be found in the "rsc" directory, and may be overridden providing a file path via the "--stylesheet" command-line argument.

## Issues and limitations

* Only one instance of the application can be running at one time.
* Certain actions, such as related candidates removal on parent series edit, are not rolled back on later failure.
* Global search may not work with pasted text if a character has not been manually entered or deleted at least once.

## Dependencies

* mingw-w64-x86_64-gcc
* mingw-w64-x86_64-pkgconf
* mingw-w64-x86_64-gtk3
