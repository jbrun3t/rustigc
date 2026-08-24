# `timezones.utz`

Zone boundaries for `decode::datetime`, lat/lon -> IANA name. Generated with `utz_build_cli`, whose
version tag carries the timezone-boundary-builder release:

```
cargo install utz_build_cli
utz_build_cli gen all 10000 --qbits 16 --grid-deg 2 --w-min 0.001 --codec none -o timezones.utz
```

It is vendored rather than generated in a `build.rs` because `utz_build` fetches the boundary data
over the network, which a published crate cannot do.

## Why not a `utz` preset

Every preset but `accurate` (8.3 MB) is built from the `now` zone set, which merges zones whose
rules agree *from the dataset's own build forward* and answers with one representative. That is
problematic for us in two ways: the name belongs to a neighbour, and the offset only holds for
flights of the dataset's era. `compact` and `balanced` do not help — they refine borders over
the same merge.

This recipe is `tiny`'s with the dataset swapped for `all`. The residual 0.8% is border error
from the 10 km simplification, flat across every eras.
