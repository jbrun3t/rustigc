# IGC specification errors seen from recorders

Real-world quirks the parser has had to absorb, with the recorders they were seen on.

## Gracefully recovered

### Undocumented header origin
> HSCCLCOMPETITION CLASS:FAI-3
* XCTrack Version ??

### Crazy carriage return
> HFDTE170717\r\r\n
* SkyLogger (Iphone)
* Flymaster NavSD
* Flymaster GpsSD
* Flytec,6015

### Space after Fix
> B1042394533160N00606294EA0078000894 \r\r\n
* Flymaster NavSD
* Flymaster GpsSD

### Missing new line at end of file
* Many ...

### Missing task header
* Many ...

### Missing Recorder ID
> AXSR
* Syride, SYS'GPS
* Syride, SYS'Nav

### Missing/Bad A Record
* Syride, SYS-Ky
* FLYTEC,6020

## Bad fix skipped

### Invalid coordinates
> B1257364513713N00560000EA0000002039
* French CFD Server 2.1
* XCTrack Version ??
* Garmin Forerunner205 ?

### Invalid timestamps
> B1449604549859N00636577EA017540175401808
* XCSoar Version ??

### Duplicate fixes, repeated timestamp
> B1108224358685N00628735EA0156801543
> B1108224358685N00628735EA0156801543
* REVERSALE VGP2010

### Longitude degree with 2 digit instead of 3 for task TP
> C4551116N0613229EB01
* REVERSALE VGP2010

### Random missing characters
> B133757440620839EA0188802041000
* BRAUNIGER,COMPETINO
