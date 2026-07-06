//! Pure milestone rules — the thresholds and bit layout, with no I/O and no
//! interior mutability.
//!
//! [`lib`](crate) owns the atomics and the hook wiring; this module owns *what
//! counts as an achievement*. Keeping it pure means the rules are unit-testable
//! in isolation and the tick-path code that calls [`earned`] stays trivial.

/// A single player achievement the module tracks. Each maps to one distinct bit
/// in the module's `unlocked` mask (via [`Milestone::bit`]), so a player's whole
/// set is one `u32` and "already unlocked?" is a single AND.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Milestone {
    /// Logged in at least once.
    FirstLogin,
    /// Reached character level 10.
    Reached10,
    /// Reached character level 20.
    Reached20,
    /// Entered enough zones to count as an explorer.
    Explorer,
    /// Sent enough chat lines to count as a chatterbox.
    Chatterbox,
    /// Used enough items to count as a tinkerer.
    Tinkerer,
}

impl Milestone {
    /// Every milestone, in bit order. Lets a caller walk the newly-set bits of a
    /// mask back to their [`Milestone`] (for logging) without a hand-kept list.
    pub const ALL: [Milestone; 6] = [
        Milestone::FirstLogin,
        Milestone::Reached10,
        Milestone::Reached20,
        Milestone::Explorer,
        Milestone::Chatterbox,
        Milestone::Tinkerer,
    ];

    /// This milestone's unique bit in the `unlocked` mask. Bits follow
    /// declaration order (`FirstLogin` = bit 0), so [`ALL`](Self::ALL), the enum,
    /// and the mask all stay in lockstep.
    #[must_use]
    pub const fn bit(self) -> u32 {
        1u32 << (self as u32)
    }

    /// A short human label for logs and operator tooling.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Milestone::FirstLogin => "First Login",
            Milestone::Reached10 => "Reached Level 10",
            Milestone::Reached20 => "Reached Level 20",
            Milestone::Explorer => "Explorer",
            Milestone::Chatterbox => "Chatterbox",
            Milestone::Tinkerer => "Tinkerer",
        }
    }
}

/// Logins needed for [`Milestone::FirstLogin`].
const FIRST_LOGIN_LOGINS: u64 = 1;
/// Highest level needed for [`Milestone::Reached10`].
const REACHED_10_LEVEL: u64 = 10;
/// Highest level needed for [`Milestone::Reached20`].
const REACHED_20_LEVEL: u64 = 20;
/// Zone entries needed for [`Milestone::Explorer`].
const EXPLORER_ZONES: u64 = 5;
/// Chat lines needed for [`Milestone::Chatterbox`].
const CHATTERBOX_LINES: u64 = 50;
/// Item uses needed for [`Milestone::Tinkerer`].
const TINKERER_USES: u64 = 10;

/// A snapshot of a player's monotonic activity counters. Built from the module's
/// atomics each event; every field only ever grows, which is what makes
/// [`earned`] monotonic — a milestone, once earned, stays earned.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Player logins observed.
    pub logins: u64,
    /// Highest character level reached.
    pub highest_level: u64,
    /// Zone entries observed.
    pub zone_enters: u64,
    /// Chat lines observed.
    pub chat_lines: u64,
    /// Item uses observed.
    pub item_uses: u64,
}

/// The bitmask of every milestone `counts` currently satisfies.
///
/// Pure and monotonic in each field: because [`Counts`] only grows, the returned
/// mask only gains bits over a session. Callers OR this into a stored mask, so
/// recomputing from scratch on each event is correct and idempotent.
#[must_use]
pub fn earned(counts: &Counts) -> u32 {
    let mut mask = 0;
    if counts.logins >= FIRST_LOGIN_LOGINS {
        mask |= Milestone::FirstLogin.bit();
    }
    if counts.highest_level >= REACHED_10_LEVEL {
        mask |= Milestone::Reached10.bit();
    }
    if counts.highest_level >= REACHED_20_LEVEL {
        mask |= Milestone::Reached20.bit();
    }
    if counts.zone_enters >= EXPLORER_ZONES {
        mask |= Milestone::Explorer.bit();
    }
    if counts.chat_lines >= CHATTERBOX_LINES {
        mask |= Milestone::Chatterbox.bit();
    }
    if counts.item_uses >= TINKERER_USES {
        mask |= Milestone::Tinkerer.bit();
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mask with every milestone bit set.
    const ALL_BITS: u32 = (1u32 << Milestone::ALL.len()) - 1;

    #[test]
    fn bits_are_distinct_single_bits() {
        let mut seen = 0u32;
        for m in Milestone::ALL {
            let b = m.bit();
            assert_eq!(b.count_ones(), 1, "{} is one bit", m.label());
            assert_eq!(seen & b, 0, "{} bit is unique", m.label());
            seen |= b;
        }
        assert_eq!(seen, ALL_BITS);
    }

    #[test]
    fn labels_are_present_and_distinct() {
        let labels: Vec<_> = Milestone::ALL.iter().map(|m| m.label()).collect();
        assert!(labels.iter().all(|l| !l.is_empty()));
        for (i, a) in labels.iter().enumerate() {
            for b in &labels[i + 1..] {
                assert_ne!(a, b, "labels are unique");
            }
        }
    }

    #[test]
    fn zeroed_counts_earn_nothing() {
        assert_eq!(earned(&Counts::default()), 0);
    }

    #[test]
    fn first_login_boundary() {
        let below = Counts {
            logins: FIRST_LOGIN_LOGINS - 1,
            ..Counts::default()
        };
        assert_eq!(earned(&below), 0);
        let at = Counts {
            logins: FIRST_LOGIN_LOGINS,
            ..Counts::default()
        };
        assert_eq!(earned(&at), Milestone::FirstLogin.bit());
    }

    #[test]
    fn level_milestones_are_cumulative() {
        let just_below = Counts {
            highest_level: REACHED_10_LEVEL - 1,
            ..Counts::default()
        };
        assert_eq!(earned(&just_below) & Milestone::Reached10.bit(), 0);

        let at_10 = Counts {
            highest_level: REACHED_10_LEVEL,
            ..Counts::default()
        };
        assert_ne!(earned(&at_10) & Milestone::Reached10.bit(), 0);
        assert_eq!(earned(&at_10) & Milestone::Reached20.bit(), 0);

        let at_20 = Counts {
            highest_level: REACHED_20_LEVEL,
            ..Counts::default()
        };
        assert_ne!(earned(&at_20) & Milestone::Reached10.bit(), 0);
        assert_ne!(earned(&at_20) & Milestone::Reached20.bit(), 0);
    }

    #[test]
    fn activity_milestones_boundaries() {
        let below = Counts {
            zone_enters: EXPLORER_ZONES - 1,
            chat_lines: CHATTERBOX_LINES - 1,
            item_uses: TINKERER_USES - 1,
            ..Counts::default()
        };
        assert_eq!(earned(&below), 0);

        let at = Counts {
            zone_enters: EXPLORER_ZONES,
            chat_lines: CHATTERBOX_LINES,
            item_uses: TINKERER_USES,
            ..Counts::default()
        };
        let mask = earned(&at);
        assert_ne!(mask & Milestone::Explorer.bit(), 0);
        assert_ne!(mask & Milestone::Chatterbox.bit(), 0);
        assert_ne!(mask & Milestone::Tinkerer.bit(), 0);
    }

    #[test]
    fn fully_satisfied_counts_earn_every_bit() {
        let maxed = Counts {
            logins: FIRST_LOGIN_LOGINS,
            highest_level: REACHED_20_LEVEL,
            zone_enters: EXPLORER_ZONES,
            chat_lines: CHATTERBOX_LINES,
            item_uses: TINKERER_USES,
        };
        assert_eq!(earned(&maxed), ALL_BITS);
    }

    #[test]
    fn earned_only_gains_bits_as_counts_grow() {
        let mut prev = 0u32;
        for n in 0..=CHATTERBOX_LINES {
            let mask = earned(&Counts {
                logins: n,
                highest_level: n,
                zone_enters: n,
                chat_lines: n,
                item_uses: n,
            });
            assert_eq!(mask & prev, prev, "bits never clear as counts grow");
            prev |= mask;
        }
        assert_eq!(prev, ALL_BITS);
    }
}
