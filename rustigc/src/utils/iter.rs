// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Iterator adapters over slices.

/// Consecutive pairs of `s`; `wraps` adds the last-to-first one, closing a circuit.
pub fn pairs<T>(s: &[T], wraps: bool) -> impl Iterator<Item = (&T, &T)> + '_ {
    s.iter().zip(s.iter().cycle().skip(1)).take(if wraps {
        s.len()
    } else {
        s.len().saturating_sub(1)
    })
}
