# rustigc-test-data

These are rustigc main test files used by integration and unit tests of the different
crates provided by rustigc.

It ships with set IGC reference tracks along with theirs scoring references. In general
these reference should not move, unless a bug is found in the scoring algorithm or the
implementation of a league is improved.

Another source of change may be a change the scoring window, the takeoff and landing,
which have an effect on the on score, depending of the scored track.

