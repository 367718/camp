# rin

Configuration files and command-line arguments.

## Behavior

Changes are not commited to disk automatically. The "save" method must be called manually.

Both the configuration and command-line arguments functionality may be used independently of each other, and every parameter is available in both. If used in conjunction, a bool is provided to signal a preference for the argument value when retrieving a specific parameter.

Duplicated fields or command-line arguments will be ignored.

Additional command-line flags and key-value pairs can be specified at initialization time. These arguments are not validated and its values are stored as UTF-8 strings.

Available modules:

* Window
  * **maximized**: bool.
  * **width**: higher-than-zero width dimension.
  * **height**: higher-than-zero height dimension.
  * **x**: higher-than-or-equal-to-zero x coordinate.
  * **y**: higher-than-or-equal-to-zero y coordinate.

* Media
  * **player**: non-empty, no-linebreak-containing UTF-8 string.
  * **iconify**: bool.
  * **flag**: non-empty, no-linebreak-containing UTF-8 string.
  * **timeout**: non-zero, non-higher-than-86400 duration in seconds.
  * **autoselect**: bool.
  * **lookup**: non-empty, no-linebreak-containing UTF-8 string.
  * **bind**: non-empty, no-linebreak-containing UTF-8 string.

* Paths
  * **files**: UTF-8 representable and no-linebreak-containing filepath.
  * **downloads**: UTF-8 representable and no-linebreak-containing filepath.
  * **pipe**: UTF-8 representable and no-linebreak-containing filepath.
  * **database**: UTF-8 representable and no-linebreak-containing filepath.

## Issues and limitations

* Command-line argument keys are case-insensitive and must start with "--", while values cannot start with "--".
* Non-UTF-8 but still valid values, such as certain filepaths, will be rejected.
