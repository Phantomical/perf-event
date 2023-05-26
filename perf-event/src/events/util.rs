use std::num::ParseIntError;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{fmt, io};

/// Helper to read and cache the type of a dynamic perf PMU event.
///
/// To get the type of a dynamic PMU event you must read its type from the file
/// at `/sys/bus/event_source/devices/<pmu>/type`. However, the type of the PMU
/// doesn't change so it is better to cache it insted of reading a file every
/// time.
///
/// See the implementation of the kprobe and uprobe events for an example of how
/// to use this type.
pub(in crate::events) struct CachedPmuType {
    name: &'static str,
    value: AtomicU32,
}

impl CachedPmuType {
    /// Create a new `CachedPmuType` from a PMU name.
    ///
    /// By default `get()` will look at
    /// `/sys/bus/event_source/devices/<pmu>/type` but you can also provide an
    /// absolute path and, in that case, it will look at `<pmu>/type`.
    pub const fn new(pmu: &'static str) -> Self {
        Self {
            name: pmu,
            // Dynamic PMUs should never have a type of 0 since that is used for the built-in
            // hardware events. We use 0 here to indicate that the type has not been initialized.
            value: AtomicU32::new(0),
        }
    }

    /// Read the type of this PMU.
    ///
    /// Will use the cached value if there is one and will read the value out of
    /// the filesystem otherwise.
    ///
    /// # Errors
    /// - Returns any IO errors from opening and reading the file.
    /// - If the type file is not able to be parsed as an integer then this
    ///   method will return an error with [`io::ErrorKind::Other`].
    pub fn get(&self) -> io::Result<u32> {
        match self.value.load(Ordering::Relaxed) {
            0 => self.read(),
            ty => Ok(ty),
        }
    }

    #[cold]
    fn read(&self) -> io::Result<u32> {
        let mut path = Path::new("/sys/bus/event_source/devices").to_path_buf();
        path.push(self.name);
        path.push("type");

        let ty = std::fs::read_to_string(&path)?
            .trim_end()
            .parse()
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    ParsePmuTypeError {
                        name: self.name,
                        error: e,
                    },
                )
            })?;

        self.value.store(ty, Ordering::Relaxed);
        Ok(ty)
    }
}

#[derive(Debug, Clone)]
struct ParsePmuTypeError {
    name: &'static str,
    error: ParseIntError,
}

impl fmt::Display for ParsePmuTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type file for pmu `{}` contained invalid data",
            self.name
        )
    }
}

impl std::error::Error for ParsePmuTypeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
