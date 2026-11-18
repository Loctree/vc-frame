//! Which terminals are producing output right now.
//!
//! Screen sees every chunk a pty delivers. This module turns that stream into
//! one native fact per terminal, published as `PaneInfo::has_output_activity`:
//! the pane has been printing without a pause for a while and has not gone
//! quiet since. It says nothing about what runs in the pane or why it prints.
//!
//! A single redraw, a keystroke echo or a dashboard that repaints once a
//! second is not activity: output counts only when it continues without a gap
//! of [`BURST_GAP`] for at least [`SUSTAINED_FOR`].

use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

/// A silence this long ends a burst; the next chunk starts a new one.
const BURST_GAP: Duration = Duration::from_millis(800);
/// How long a burst must last before the terminal is announced as active.
const SUSTAINED_FOR: Duration = Duration::from_millis(1500);
/// How long after its last chunk an announced terminal stays announced.
const QUIET_AFTER: Duration = Duration::from_millis(3000);
/// A terminal silent this long is forgotten, whether or not it still exists.
const FORGET_AFTER: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy)]
struct Burst {
    started: Instant,
    last: Instant,
}

#[derive(Debug, Default)]
pub(crate) struct OutputActivity {
    bursts: HashMap<u32, Burst>,
    announced: BTreeSet<u32>,
}

impl OutputActivity {
    /// Record one chunk of pty output for a terminal.
    pub(crate) fn note_output(&mut self, terminal_id: u32, now: Instant) {
        self.bursts
            .entry(terminal_id)
            .and_modify(|burst| {
                if now.saturating_duration_since(burst.last) >= BURST_GAP {
                    burst.started = now;
                }
                burst.last = now;
            })
            .or_insert(Burst {
                started: now,
                last: now,
            });
    }

    /// Re-evaluate every terminal against the clock. Returns true when the set
    /// of announced terminals changed, so the caller republishes pane state
    /// on a change and stays silent otherwise.
    pub(crate) fn refresh(&mut self, now: Instant) -> bool {
        self.bursts
            .retain(|_, burst| now.saturating_duration_since(burst.last) < FORGET_AFTER);
        let announced: BTreeSet<u32> = self
            .bursts
            .iter()
            .filter(|(_, burst)| {
                burst.last.saturating_duration_since(burst.started) >= SUSTAINED_FOR
                    && now.saturating_duration_since(burst.last) < QUIET_AFTER
            })
            .map(|(terminal_id, _)| *terminal_id)
            .collect();
        let changed = announced != self.announced;
        self.announced = announced;
        changed
    }

    pub(crate) fn is_announced(&self, terminal_id: u32) -> bool {
        self.announced.contains(&terminal_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// Feed chunks every `step` from `from` up to and including `to`.
    fn feed(activity: &mut OutputActivity, id: u32, start: Instant, from: u64, to: u64, step: u64) {
        let mut at = from;
        while at <= to {
            activity.note_output(id, start + ms(at));
            at += step;
        }
    }

    #[test]
    fn continuous_output_is_announced_and_silence_withdraws_it() {
        let start = Instant::now();
        let mut activity = OutputActivity::default();
        feed(&mut activity, 7, start, 0, 2000, 100);

        assert!(activity.refresh(start + ms(2100)));
        assert!(activity.is_announced(7));
        // Nothing changed on the next tick: the caller must not republish.
        assert!(!activity.refresh(start + ms(3100)));
        assert!(activity.is_announced(7));

        assert!(activity.refresh(start + ms(2000) + QUIET_AFTER));
        assert!(!activity.is_announced(7));
    }

    #[test]
    fn a_single_redraw_or_keystroke_echo_is_not_activity() {
        let start = Instant::now();
        let mut activity = OutputActivity::default();
        activity.note_output(7, start);
        activity.note_output(7, start + ms(40));

        assert!(!activity.refresh(start + ms(100)));
        assert!(!activity.is_announced(7));
    }

    #[test]
    fn a_dashboard_that_repaints_once_a_second_is_not_activity() {
        let start = Instant::now();
        let mut activity = OutputActivity::default();
        feed(&mut activity, 7, start, 0, 30_000, 1000);

        assert!(!activity.refresh(start + ms(30_100)));
        assert!(!activity.is_announced(7));
    }

    #[test]
    fn a_pause_restarts_the_burst() {
        let start = Instant::now();
        let mut activity = OutputActivity::default();
        feed(&mut activity, 7, start, 0, 1000, 100);
        // One second of silence, then a fresh second of output: neither half
        // is sustained on its own.
        feed(&mut activity, 7, start, 2000, 3000, 100);

        assert!(!activity.refresh(start + ms(3100)));
        assert!(!activity.is_announced(7));
    }

    #[test]
    fn terminals_are_tracked_independently_and_forgotten_when_long_silent() {
        let start = Instant::now();
        let mut activity = OutputActivity::default();
        feed(&mut activity, 1, start, 0, 2000, 100);
        activity.note_output(2, start + ms(2000));

        assert!(activity.refresh(start + ms(2100)));
        assert!(activity.is_announced(1));
        assert!(!activity.is_announced(2));

        assert!(activity.refresh(start + ms(2000) + FORGET_AFTER));
        assert!(!activity.is_announced(1));
        assert!(activity.bursts.is_empty());
    }
}
