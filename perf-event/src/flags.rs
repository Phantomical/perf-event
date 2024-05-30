use bitflags::bitflags;

use crate::sys::bindings;
use crate::{Builder, ReadFormat, SampleFlag};

used_in_docs!(Builder);
used_in_docs!(SampleFlag);

pub(crate) trait ReadFormatExt: Sized {
    const MAX_NON_GROUP_SIZE: usize;
    fn prefix_len(&self) -> usize;
    fn element_len(&self) -> usize;
}

impl ReadFormatExt for ReadFormat {
    const MAX_NON_GROUP_SIZE: usize = Self::all() //
        .difference(Self::GROUP)
        .bits()
        .count_ones() as usize
        + 1;

    // The format of a read from a group is like this
    // struct read_format {
    //     u64 nr;            /* The number of events */
    //     u64 time_enabled;  /* if PERF_FORMAT_TOTAL_TIME_ENABLED */
    //     u64 time_running;  /* if PERF_FORMAT_TOTAL_TIME_RUNNING */
    //     struct {
    //         u64 value;     /* The value of the event */
    //         u64 id;        /* if PERF_FORMAT_ID */
    //         u64 lost;      /* if PERF_FORMAT_LOST */
    //     } values[nr];
    // };

    /// The size of the common prefix when reading a group.
    fn prefix_len(&self) -> usize {
        1 + (*self & (Self::TOTAL_TIME_ENABLED | Self::TOTAL_TIME_RUNNING))
            .bits()
            .count_ones() as usize
    }

    /// The size of each element when reading a group
    fn element_len(&self) -> usize {
        1 + (*self & (Self::ID | Self::LOST)).bits().count_ones() as usize
    }
}

/// Configuration of how much skid is allowed when gathering samples.
///
/// Skid is the number of instructions that occur between an event occuring and
/// a sample being gathered by the kernel. Less skid is better but there are
/// hardware limitations around how small the skid can be.
///
/// Also see [`Builder::precise_ip`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SampleSkid {
    /// There may be an arbitrary number of instructions between the event and
    /// the recorded instruction pointer.
    Arbitrary = 0,

    /// There may be a constant number of instructions between the event and
    /// and the recorded instruction pointer.
    Constant = 1,

    /// We've requested that there be 0 skid. This does not guarantee that
    /// samples will actually have 0 skid.
    RequestZero = 2,

    /// Skid must be 0. If skid is 0 then the generated sample records will
    /// have the `PERF_RECORD_MISC_EXACT_IP` bit set.
    RequireZero = 3,
}

/// Supported linux clocks that can be used within a perf_event instance.
///
/// See the [`clock_gettime(2)`][0] manpage for the full documentation on what
/// each clock value actually means.
///
/// [0]: https://www.mankier.com/2/clock_gettime
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Clock(libc::clockid_t);

impl Clock {
    /// A clock following International Atomic Time.
    pub const TAI: Self = Self::new(libc::CLOCK_TAI);

    /// A clock that measures wall-clock time.
    pub const REALTIME: Self = Self::new(libc::CLOCK_REALTIME);

    /// A clock that is identical to `MONOTONIC` except it also includes any
    /// time during which the systems was suspended.
    pub const BOOTTIME: Self = Self::new(libc::CLOCK_BOOTTIME);

    /// A clock that (roughly) corresponds to the time that the system has been
    /// running since it was booted. (On Linux, at least).
    pub const MONOTONIC: Self = Self::new(libc::CLOCK_MONOTONIC);

    /// Similar to `MONOTONIC` but does not include NTP adjustments.
    pub const MONOTONIC_RAW: Self = Self::new(libc::CLOCK_MONOTONIC_RAW);
}

impl Clock {
    /// Construct a new `Clock` from the libc clockid value.
    pub const fn new(clockid: libc::clockid_t) -> Self {
        Self(clockid)
    }

    /// Extract the libc clockid value.
    pub const fn into_raw(self) -> libc::clockid_t {
        self.0
    }
}

bitflags! {
    /// Specify what branches to include in a branch record.
    ///
    /// This is used by the builder in combination with setting
    /// [`SampleFlag::BRANCH_STACK`].
    ///
    /// The first part of the value is the privilege level, which is a
    /// combination of `USER`, `BRANCH`, or `HV`. `PLM_ALL` is a convenience
    /// value with all 3 ORed together. If none of the privilege levels are set
    /// then the kernel will use the privilege level of the event.
    ///
    /// The second part specifies which branch types are to be included in the
    /// branch stack. At least one of these bits must be set.
    pub struct SampleBranchFlag: u64 {
        /// The branch target is in user space.
        const USER = bindings::PERF_SAMPLE_BRANCH_USER as _;

        /// The branch target is in kernel space.
        const KERNEL = bindings::PERF_SAMPLE_BRANCH_KERNEL as _;

        /// The branch target is in the hypervisor.
        const HV = bindings::PERF_SAMPLE_BRANCH_HV as _;

        /// Include any branch type.
        const ANY = bindings::PERF_SAMPLE_BRANCH_ANY as _;

        /// Include any call branch.
        ///
        /// This includes direct calls, indirect calls, and far jumps.
        const ANY_CALL = bindings::PERF_SAMPLE_BRANCH_ANY_CALL as _;

        /// Include indirect calls.
        const IND_CALL = bindings::PERF_SAMPLE_BRANCH_IND_CALL as _;

        /// Include direct calls.
        const CALL = bindings::PERF_SAMPLE_BRANCH_CALL as _;

        /// Include any return branch.
        const ANY_RETURN = bindings::PERF_SAMPLE_BRANCH_ANY_RETURN as _;

        /// Include indirect jumps.
        const IND_JUMP = bindings::PERF_SAMPLE_BRANCH_IND_JUMP as _;

        /// Include conditional branches.
        const COND = bindings::PERF_SAMPLE_BRANCH_COND as _;

        /// Include transactional memory aborts.
        const ABORT_TX = bindings::PERF_SAMPLE_BRANCH_ABORT_TX as _;

        /// Include branches in a transactional memory transaction.
        const IN_TX = bindings::PERF_SAMPLE_BRANCH_IN_TX as _;

        /// Include branches not in a transactional memory transaction.
        const NO_TX = bindings::PERF_SAMPLE_BRANCH_NO_TX as _;

        /// Include branches that are part of a hardware-generated call stack.
        ///
        /// Note that this requires hardware support. See the [manpage][0] for
        /// platforms which support this.
        ///
        /// [0]: https://www.mankier.com/2/perf_event_open
        const CALL_STACK = bindings::PERF_SAMPLE_BRANCH_CALL_STACK as _;
    }
}

impl SampleBranchFlag {
    /// All privilege levels (`USER`, `KERNEL`, and `HV`) ORed together.
    pub const PLM_ALL: Self = Self::USER.union(Self::KERNEL).union(Self::HV);
}
