use bytes::Buf;

use super::{RecordEvent, RecordMiscFlags};

/// SWITCH_CPU_WIDE records indicates a context switch when profiling in
/// cpu-wide mode.
///
/// It provides some additional info on the process being switched that is not
/// provided by [`Switch`].
///
/// This enum corresponds to `PERF_RECORD_SWITCH_CPU_WIDE`. See the [manpage]
/// for more documentation.
///
/// [`Switch`]: crate::samples::RecordEvent::Switch
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Copy, Clone, Debug)]
pub enum SwitchCpuWide {
    /// A process thread was switched into the monitored CPU.
    SwitchIn {
        /// The process ID of the incoming process.
        next_pid: u32,

        /// The thread ID of the incoming process thread.
        next_tid: u32,
    },

    /// A process thread was switched out of the monitored CPU.
    SwitchOut {
        /// The process ID of the outgoing process.
        prev_pid: u32,

        /// The thread ID of the outgoing process thread.
        prev_tid: u32,
    },
}

impl SwitchCpuWide {
    /// Get the process ID associated with the context switch.
    ///
    /// Depending on whether this is a switch-in or a switch-out this will be
    /// the incoming process ID or outgoing process ID, respectively.
    pub fn pid(&self) -> u32 {
        match *self {
            Self::SwitchIn { next_pid, .. } => next_pid,
            Self::SwitchOut { prev_pid, .. } => prev_pid,
        }
    }

    /// Get the thread ID associated with the context switch.
    ///
    /// Depending on whether this is a switch-in or a switch-out this will be
    /// the incoming thread ID or the outgoing thread ID, respectively.
    pub fn tid(&self) -> u32 {
        match *self {
            Self::SwitchIn { next_tid, .. } => next_tid,
            Self::SwitchOut { prev_tid, .. } => prev_tid,
        }
    }

    pub(crate) fn parse<B: Buf>(misc: RecordMiscFlags, buf: &mut B) -> Self {
        let pid = buf.get_u32_ne();
        let tid = buf.get_u32_ne();

        if misc.contains(RecordMiscFlags::SWITCH_OUT) {
            Self::SwitchOut {
                prev_pid: pid,
                prev_tid: tid,
            }
        } else {
            Self::SwitchIn {
                next_pid: pid,
                next_tid: tid,
            }
        }
    }
}

impl From<SwitchCpuWide> for RecordEvent {
    fn from(switch: SwitchCpuWide) -> Self {
        Self::SwitchCpuWide(switch)
    }
}
