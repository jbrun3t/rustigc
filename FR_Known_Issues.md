# Known Issues:

* Longitude degree with 2 digit instead of 3 for task TP
  > C4551116N0613229EB01
  * REVERSALE VGP2010

* Minutes coordinates out of bounds (>=60)
  > B1257364513713N00560000EA0000002039
  * French CFD Server 2.1
  * XCTrack Version ??

* Undocumented header origin:
  > HSCCLCOMPETITION CLASS:FAI-3
  * XCTrack Version ??

* Crazy carriage return
  HFDTE170717\r\r\n
  * SkyLogger (Iphone)
  * Flymaster NavSD
  * Flymaster GpsSD
  * Flytec,6015

* Space after Fix
  > B1042394533160N00606294EA0078000894 \r\r\n
  * Flymaster NavSD
  * Flymaster GpsSD

* Missing new line at end of file
  * Flymaster NavSD (XLF) / Logfly version 3->4.1.7
  * Many ...

* Missing Recorder ID
  > AXSR
  * Syride, SYS'GPS
  * Syride, SYS'Nav

* Missing A Record
  * Syride, SYS-Ky

* Missing task declaration
  * BRAUNIGER,COMPEO+

