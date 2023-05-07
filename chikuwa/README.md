# chikuwa

Collection of small utilities.

## Behavior

Available functionality:

* **EphemeralPath**: Paths that attempt to delete themselves when dropped.
* **concat_str**: Macro for efficient string concatenation.
* **register_app**: Attempt to prevent more than one instance of an application from running.
* **execute_app**: Run command or open associated external application.
* **current_date**: Current date as string in YYYYMMDD format.
* **percent_encode**: Replace certain characters of a string with a safe representation for inclusion in a URL.
* **natural_cmp**: String comparison that takes numerical values into consideration.
* **case_insensitive_contains**: Whether a list of strings are all case-insensitively contained in a string.