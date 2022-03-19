## Message Size Distribution
The distribution of the total size of a message.
```
|-- 114234519 bytes -------------|
|                                |
|-- Type --|------- Data --------|
|          |                     |
|- 1 byte -|-- 114234518 bytes --|
```
----
## Message Types
What message types exist and what are their binary identifiers.
```
Hex |-- Bytes --| Name
#00 | 0000 0000 | ...
#01 | 0000 0001 | ...
#02 | 0000 0010 | ...
#03 | 0000 0011 | ...
```
----
## Message Formating
What parameters are available for each message type.
```
|----- Type -----| Parameters
...              | ...
...              | ...
...              | ...
...              | ...
```