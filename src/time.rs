//! Helpers for working with smoltcp's time utilities inside uefi.

use smoltcp::time::Instant;
use uefi::proto::misc::Timestamp;

/// Helper for generating [`smoltcp::time::Instant`]s using an underlying
/// UEFI [`Timestamp`] protocol utility.
///
/// # Known limitations
///
/// This may overflow and break monotonicity.
pub struct TimestampClock<'a> {
    ts: &'a Timestamp,
    frequency: u64,
}

impl<'a> TimestampClock<'a> {
    pub fn new(ts: &'a Timestamp) -> Result<Self, uefi::Error> {
        let props = ts.get_properties()?;
        Ok(Self {
            ts,
            frequency: props.frequency,
        })
    }

    pub fn now(&self) -> Instant {
        let micros = self.ts.get_timestamp() * 1000000 / self.frequency;
        Instant::from_micros(micros as i64)
    }
}

/// Returns a [smoltcp::time::Instant] with microseconds equal to the
/// number of processor clocks.
///
/// Please note the name; this is extremely stupid in a number of different ways,
/// but all that smoltcp appears to care about is monotonicity and this will provide it :)
///
/// If your UEFI environment does not appear to provide a [Timestamp]
/// protocol facility that you can put in [TimestampClock], this can be used,
/// but you really really should use [TimestampClock] instead.
///
/// # Panics
///
/// When not called on a supported platform. Supported platforms are currently
/// x86_64 and aarch64.
///
/// # Known limitations
///
/// - This may overflow early and break monotonicity.
/// - This will not be monotonic if called from different cores.
pub fn shitty_now_from_processor_clock() -> Instant {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        return Instant::from_micros(core::arch::x86_64::_rdtsc() as i64);
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut ticks: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) ticks);
        Instant::from_micros(ticks as i64)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    panic!("shitty_now_from_processor_clock is not implemented for this platform!");
}
