# chiaki

Database operations.

## Behavior

Changes are not committed to disk automatically. The "save" method must be called manually.

Available modules:

* Feeds
	* **url**: non-empty, unique UTF-8 string.

* Formats
	* **name**: non-empty, unique UTF-8 string.

* Kinds
	* **name**: non-empty, unique UTF-8 string.

* Series
	* **title**: non-empty, unique UTF-8 string.
	* **kind**: defined Kind ID.
	* **status**: "Watching"/"OnHold"/"PlanToWatch"/"Completed" enum. If a related Candidate is defined, can only be set to "Watching".
	* **progress**: non-zero (if status is "Watching", "OnHold" or "Completed") or zero (if status is "PlanToWatch") number.
	* **good**: "Yes"/"No" enum. Can only be set to "Yes" if status is "Completed".

* Candidates
 	* **series**: defined, unique and status "Watching" Series ID.
 	* **title**: non-empty, unique UTF-8 string.
 	* **group**: UTF-8 string.
 	* **quality**: UTF-8 string.
 	* **offset**: not-greater-than-99999 number.
 	* **current**: "Yes"/"No" enum.
 	* **downloaded**: non-zero number set. Empty if current is "No".

## Issues and limitations

* Concurrent use of this library is discouraged.
* Database size is limited to 50 Megabytes.
