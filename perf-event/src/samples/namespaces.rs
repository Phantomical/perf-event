use bytes::Buf;
use perf_event_open_sys::bindings;

use super::{Parse, RecordEvent};

/// NAMESPACES records include namespace information of a process.
///
/// This struct corresponds to `PERF_RECORD_NAMESPACES`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Namespaces {
    /// Process ID.
    pub pid: u32,

    /// Thread ID.
    pub tid: u32,

    /// Entries for various namespaces.
    ///
    /// Specific namespaces have fixed indices within this array. Accessors
    /// have been provided for some of these. See the [manpage] for the full
    /// documentation.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub namespaces: Vec<NamespaceEntry>,
}

impl Namespaces {
    /// Network namepsace
    pub fn network(&self) -> Option<NamespaceEntry> {
        self.namespaces
            .get(bindings::NET_NS_INDEX as usize)
            .copied()
    }

    /// UTS namespace.
    pub fn uts(&self) -> Option<NamespaceEntry> {
        self.namespaces
            .get(bindings::USER_NS_INDEX as usize)
            .copied()
    }

    /// IPC namespace.
    pub fn ipc(&self) -> Option<NamespaceEntry> {
        self.namespaces
            .get(bindings::IPC_NS_INDEX as usize)
            .copied()
    }

    /// PID namespace.
    pub fn pid(&self) -> Option<NamespaceEntry> {
        self.namespaces
            .get(bindings::PID_NS_INDEX as usize)
            .copied()
    }

    /// User namespace.
    pub fn user(&self) -> Option<NamespaceEntry> {
        self.namespaces
            .get(bindings::USER_NS_INDEX as usize)
            .copied()
    }

    /// Cgroup namespace.
    pub fn cgroup(&self) -> Option<NamespaceEntry> {
        self.namespaces
            .get(bindings::CGROUP_NS_INDEX as usize)
            .copied()
    }
}

/// An individual namespace entry.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub struct NamespaceEntry {
    pub dev: u64,
    pub inode: u64,
}

impl Parse for Namespaces {
    fn parse<B: Buf>(config: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            pid: buf.get_u32_ne(),
            tid: buf.get_u32_ne(),
            namespaces: {
                let len = buf.get_u64_ne() as usize;

                std::iter::repeat_with(|| NamespaceEntry::parse(config, buf))
                    .take(len)
                    .collect()
            },
        }
    }
}

impl Parse for NamespaceEntry {
    fn parse<B: Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            dev: buf.get_u64_ne(),
            inode: buf.get_u64_ne(),
        }
    }
}

impl From<Namespaces> for RecordEvent {
    fn from(ns: Namespaces) -> Self {
        Self::Namespaces(ns)
    }
}
