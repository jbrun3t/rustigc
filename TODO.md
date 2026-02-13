# TODO:
* Improve cli tool: more option, log/raw/quiet / outformat
* Fixup altitude on fix insertion
* Improve python bindings for less copy
* Consider using the GeoPoint and FlatPoint abstraction is it help
  readability and/or performance
* Improve Analysis API
* Flight smoothing for analysis: See Ideas
* Add multi takeoff/detection, flight split (early phase detection)
* Add flight phase analysis (thermaling, transition, etc ...)
* Flight Scoring support

# IDEAS:
* Smoothing: (for flight dynamics and phases analysis - not scoring)
  - using BSpline + Outliner removal (hubert? mad? tdist? cg?)
  - Or univariate if implementation available in rust
  - possible lib: scirs2, Fear::cg,

* Phase analysis:
  - From dynamics like groud speed, vario, RoT
  - Some form of Hidden Markov Model
  - Need tools to viz and edit annotations (quick gui)
  - Start simple with Onground / Flying-Around / Transistion / Thermal
  - Supervised first pass from fixed algo (see logfly,libigc) (python learnhmm maybe ?)
  - Then unsupervised (discover new states - Dividing the Flying around one)
  - Contruct data set for future training

* Scoring:
  - Use igc-xc-score algorythm with cheap projection to discover turnpoints
  - Once TPs found, recompute distance from accurate Geodesic
  - Script to compare result igc-xc-score
