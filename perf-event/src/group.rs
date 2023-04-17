use std::fmt;
use std::io;
use std::iter::FusedIterator;
use std::time::Duration;

use crate::events::Software;
use crate::{Builder, Counter, ReadFormat};

const SKIP_GROUP: ReadFormat = ReadFormat::from_bits_retain(1 << (u64::BITS - 1));

/// A group of counters that can be managed as a unit.
///
/// A `Group` represents a group of [`Counter`]s that can be enabled,
/// disabled, reset, or read as a single atomic operation. This is necessary if
/// you want to compare counter values, produce ratios, and so on, since those
/// operations are only meaningful on counters that cover exactly the same
/// period of execution.
///
/// A `Counter` is placed in a group when it is created via the
/// [`Builder::build_with_group`] method. A `Group`'s [`read`] method returns
/// values of all its member counters at once as a [`GroupData`] value, which
/// can be indexed by `Counter` to retrieve a specific value.
///
/// The lifetime of a `Group` and its associated `Counter`s are independent:
/// you can drop them in any order and they will continue to work. A `Counter`
/// will continue to work after the `Group` is dropped. If a `Counter` is
/// dropped first then it will simply be removed from the `Group`.
///
/// Enabling or disabling a `Group` affects each `Counter` that belongs to it.
/// Subsequent reads from the `Counter` will not reflect activity while the
/// `Group` was disabled, unless the `Counter` is re-enabled individually.
///
/// ## Limits on group size
///
/// Hardware counters are implemented using special-purpose registers on the
/// processor, of which there are only a fixed number. (For example, an Intel
/// high-end laptop processor from 2015 has four such registers per virtual
/// processor.) Without using groups, if you request more hardware counters than
/// the processor can actually support, a complete count isn't possible, but the
/// kernel will rotate the processor's real registers amongst the measurements
/// you've requested to at least produce a sample.
///
/// But since the point of a counter group is that its members all cover exactly
/// the same period of time, this tactic can't be applied to support large
/// groups. If the kernel cannot schedule a group, its counters remain zero. I
/// think you can detect this situation by comparing the group's
/// [`time_enabled`] and [`time_running`] values. If the [`pinned`] option is
/// set then you will also be able to detect this by [`read`] returning an error
/// with kind [`UnexpectedEof`].
///
/// According to the `perf_list(1)` man page, you may be able to free up a
/// hardware counter by disabling the kernel's NMI watchdog, which reserves one
/// for detecting kernel hangs:
///
/// ```text
/// $ echo 0 > /proc/sys/kernel/nmi_watchdog
/// ```
///
/// You can reenable the watchdog when you're done like this:
///
/// ```text
/// $ echo 1 > /proc/sys/kernel/nmi_watchdog
/// ```
///
/// [`read`]: Self::read
/// [`pinned`]: Builder::pinned
/// [`UnexpectedEof`]: io::ErrorKind::UnexpectedEof
///
/// # Examples
/// Compute the average cycles-per-instruction (CPI) for a call to `println!`:
/// ```
/// use perf_event::events::Hardware;
/// use perf_event::{Builder, Group};
///
/// let mut group = Group::new()?;
/// let cycles = group.add(&Builder::new(Hardware::CPU_CYCLES))?;
/// let insns = group.add(&Builder::new(Hardware::INSTRUCTIONS))?;
///
/// let vec = (0..=51).collect::<Vec<_>>();
///
/// group.enable()?;
/// println!("{:?}", vec);
/// group.disable()?;
///
/// let counts = group.read()?;
/// println!(
///     "cycles / instructions: {} / {} ({:.2} cpi)",
///     counts[&cycles],
///     counts[&insns],
///     (counts[&cycles] as f64 / counts[&insns] as f64)
/// );
/// # std::io::Result::Ok(())
/// ```
///
/// [`group`]: Builder::group
/// [`read`]: Group::read
/// [`time_enabled`]: GroupData::time_enabled
/// [`time_running`]: GroupData::time_running
pub struct Group(pub(crate) Counter);

impl Group {
    /// Construct a new, empty `Group`.
    ///
    /// The resulting `Group` is only suitable for observing the current process
    /// on any CPU. If you need to build a `Group` with different settings you
    /// will need to use [`Builder::build_group`].
    pub fn new() -> io::Result<Group> {
        Builder::new(Software::DUMMY)
            .read_format(
                ReadFormat::GROUP
                    | ReadFormat::TOTAL_TIME_ENABLED
                    | ReadFormat::TOTAL_TIME_RUNNING
                    | ReadFormat::ID,
            )
            .build_group()
    }

    /// Access the internal counter for this group.
    pub fn as_counter(&self) -> &Counter {
        &self.0
    }

    /// Mutably access the internal counter for this group.
    pub fn as_counter_mut(&mut self) -> &mut Counter {
        &mut self.0
    }

    /// Convert this `Group` into its internal counter.
    pub fn into_counter(self) -> Counter {
        self.0
    }

    /// Return this group's kernel-assigned unique id.
    pub fn id(&self) -> u64 {
        self.0.id()
    }

    /// Enable all counters in this `Group`.
    pub fn enable(&mut self) -> io::Result<()> {
        self.0.enable_group()
    }

    /// Disable all counters in this `Group`
    pub fn disable(&mut self) -> io::Result<()> {
        self.0.disable_group()
    }

    /// Reset the value of all counters in this `Group` to zero.
    pub fn reset(&mut self) -> io::Result<()> {
        self.0.reset_group()
    }

    /// Construct a new counter as a part of this group.
    ///
    /// # Example
    /// ```
    /// use perf_event::events::Hardware;
    /// use perf_event::{Builder, Group};
    ///
    /// let mut group = Group::new()?;
    /// let counter = group.add(&Builder::new(Hardware::INSTRUCTIONS).any_cpu());
    /// #
    /// # std::io::Result::Ok(())
    /// ```
    pub fn add(&mut self, builder: &Builder) -> io::Result<Counter> {
        builder.build_with_group(self)
    }

    /// Return the values of all the `Counter`s in this `Group` as a [`Counts`]
    /// value.
    ///
    /// A `Counts` value is a map from specific `Counter`s to their values. You
    /// can find a specific `Counter`'s value by indexing:
    ///
    /// ```ignore
    /// let mut group = Group::new()?;
    /// let counter1 = Builder::new().group(&mut group).kind(...).build()?;
    /// let counter2 = Builder::new().group(&mut group).kind(...).build()?;
    /// ...
    /// let counts = group.read()?;
    /// println!("Rhombus inclinations per taxi medallion: {} / {} ({:.0}%)",
    ///          counts[&counter1],
    ///          counts[&counter2],
    ///          (counts[&counter1] as f64 / counts[&counter2] as f64) * 100.0);
    /// ```
    ///
    /// [`Counts`]: struct.Counts.html
    pub fn read(&mut self) -> io::Result<GroupData> {
        let mut data = self.0.read_group()?;
        data.set_skip_group();
        Ok(data)
    }
}

impl AsRef<Counter> for &'_ Group {
    fn as_ref(&self) -> &Counter {
        &self.0
    }
}

impl AsMut<Counter> for &'_ mut Group {
    fn as_mut(&mut self) -> &mut Counter {
        &mut self.0
    }
}

/// A collection of counts from a [`Group`] of counters.
///
/// This is the type returned by calling [`read`] on a [`Group`].
/// You can index it with a reference to a specific `Counter`:
///
/// ```
/// use perf_event::events::Hardware;
/// use perf_event::{Builder, Group};
///
/// let mut group = Group::new()?;
/// let cycles = group.add(&Builder::new(Hardware::CPU_CYCLES))?;
/// let insns = group.add(&Builder::new(Hardware::INSTRUCTIONS))?;
/// let counts = group.read()?;
/// println!(
///     "cycles / instructions: {} / {} ({:.2} cpi)",
///     counts[&cycles],
///     counts[&insns],
///     (counts[&cycles] as f64 / counts[&insns] as f64)
/// );
/// # std::io::Result::Ok(())
/// ```
///
/// Or you can iterate over the results it contains:
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # use perf_event::Group;
/// # let counts = Group::new()?.read()?;
/// for entry in &counts {
///     println!("Counter id {} has value {}", entry.id(), entry.value());
/// }
/// # Ok(())
/// # }
/// ```
///
/// The `id` values produced by this iteration are internal identifiers assigned
/// by the kernel. You can use the [`Counter::id`] method to find a
/// specific counter's id.
///
/// For some kinds of events, the kernel may use timesharing to give all
/// counters access to scarce hardware registers. You can see how long a group
/// was actually running versus the entire time it was enabled using the
/// `time_enabled` and `time_running` methods:
///
/// ```
/// # use perf_event::{Builder, Group};
/// # use perf_event::events::Software;
/// # let mut group = Group::new()?;
/// # let insns = group.add(&Builder::new(Software::DUMMY))?;
/// # let counts = group.read()?;
/// let scale =
///     counts.time_enabled().unwrap().as_secs_f64() / counts.time_running().unwrap().as_secs_f64();
/// for entry in &counts {
///     let value = entry.value() as f64 * scale;
///
///     print!("Counter id {} has value {}", entry.id(), value as u64);
///     if scale > 1.0 {
///         print!(" (estimated)");
///     }
///     println!();
/// }
/// # std::io::Result::Ok(())
/// ```
///
/// [`read`]: Group::read
pub struct GroupData {
    // Raw results from the `read`.
    data: Vec<u64>,
    read_format: ReadFormat,
}

impl GroupData {
    pub(crate) fn new(data: Vec<u64>, read_format: ReadFormat) -> Self {
        Self { data, read_format }
    }

    /// Return the number of counters this `Counts` holds results for.
    #[allow(clippy::len_without_is_empty)] // Groups are never empty.
    pub fn len(&self) -> usize {
        let len = self.data[0] as usize;

        if self.skip_group() {
            len - 1
        } else {
            len
        }
    }

    /// The duration for which the group was enabled.
    ///
    /// This will only be present if [`TOTAL_TIME_ENABLED`] was passed to
    /// [`read_format`]
    ///
    /// [`TOTAL_TIME_ENABLED`]: ReadFormat::TOTAL_TIME_ENABLED
    /// [`read_format`]: Builder::read_format
    pub fn time_enabled(&self) -> Option<Duration> {
        self.prefix_offset_of(ReadFormat::TOTAL_TIME_ENABLED)
            .map(|idx| self.data[idx])
            .map(Duration::from_nanos)
    }

    /// The duration for which the group was scheduled on the CPU.
    ///
    /// This will only be present if [`TOTAL_TIME_RUNNING`] was passed to
    /// [`read_format`]
    ///
    /// [`TOTAL_TIME_ENABLED`]: ReadFormat::TOTAL_TIME_RUNNING
    /// [`read_format`]: Builder::read_format
    ///
    /// Return the number of nanoseconds the `Group` was actually collecting
    /// counts that contributed to this `Counts`' contents.
    ///
    /// [`TOTAL_TIME_RUNNING`]: ReadFormat::TOTAL_TIME_RUNNING
    pub fn time_running(&self) -> Option<Duration> {
        self.prefix_offset_of(ReadFormat::TOTAL_TIME_RUNNING)
            .map(|idx| self.data[idx])
            .map(Duration::from_nanos)
    }

    /// Get the entry for `member` in `self`, or `None` if `member` is not
    /// present.
    ///
    /// `member` can be either a `Counter` or a `Group`.
    ///
    /// If you know the counter is in the group then you can access the count
    /// via indexing.
    /// ```
    /// use perf_event::events::Hardware;
    /// use perf_event::{Builder, Group};
    ///
    /// let mut group = Group::new()?;
    /// let instrs = Builder::new(Hardware::INSTRUCTIONS).build_with_group(&mut group)?;
    /// let cycles = Builder::new(Hardware::CPU_CYCLES).build_with_group(&mut group)?;
    /// group.enable()?;
    /// // ...
    /// let counts = group.read()?;
    /// let instrs = counts[&instrs];
    /// let cycles = counts[&cycles];
    /// # std::io::Result::Ok(())
    /// ```
    pub fn get(&self, member: &Counter) -> Option<GroupEntry> {
        self.iter_with_group()
            .find(|entry| entry.id() == member.id())
    }

    /// Return an iterator over all entries in `self`.
    ///
    /// For compatibility reasons, if the [`Group`] this was
    ///
    /// # Example
    /// ```
    /// # use perf_event::Group;
    /// # let mut group = Group::new()?;
    /// let data = group.read()?;
    /// for entry in &data {
    ///     println!("Counter with id {} has value {}", entry.id(), entry.value());
    /// }
    /// # std::io::Result::Ok(())
    /// ```
    pub fn iter(&self) -> GroupIter {
        let mut iter = self.iter_with_group();
        if self.skip_group() {
            let _ = iter.next();
        }
        iter
    }

    fn iter_with_group(&self) -> GroupIter {
        GroupIter::new(
            self.read_format,
            &self.data[self.read_format.prefix_len()..],
        )
    }

    fn skip_group(&self) -> bool {
        self.read_format.contains(SKIP_GROUP)
    }

    fn set_skip_group(&mut self) {
        self.read_format |= SKIP_GROUP;
    }

    fn prefix_offset_of(&self, flag: ReadFormat) -> Option<usize> {
        debug_assert_eq!(flag.bits().count_ones(), 1);

        let read_format =
            self.read_format & (ReadFormat::TOTAL_TIME_ENABLED | ReadFormat::TOTAL_TIME_RUNNING);

        if !self.read_format.contains(flag) {
            return None;
        }

        Some((read_format.bits() & (flag.bits() - 1)).count_ones() as _)
    }
}

impl std::ops::Index<&Counter> for GroupData {
    type Output = u64;

    fn index(&self, ctr: &Counter) -> &u64 {
        let data = self
            .iter_with_group()
            .iter
            .find(|data| {
                let entry = GroupEntry::new(self.read_format, *data);
                entry.id() == ctr.id()
            })
            .unwrap_or_else(|| panic!("group contained no counter with id {}", ctr.id()));

        &data[0]
    }
}

impl fmt::Debug for GroupData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct GroupEntries<'a>(&'a GroupData);

        impl fmt::Debug for GroupEntries<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        let mut dbg = fmt.debug_struct("GroupData");

        if let Some(time_enabled) = self.time_enabled() {
            dbg.field("time_enabled", &time_enabled.as_nanos());
        }

        if let Some(time_running) = self.time_running() {
            dbg.field("time_running", &time_running.as_nanos());
        }

        dbg.field("entries", &GroupEntries(self));
        dbg.finish()
    }
}

impl<'a> IntoIterator for &'a GroupData {
    type IntoIter = GroupIter<'a>;
    type Item = <GroupIter<'a> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Individual entry for a counter returned by [`Group::read`].
#[derive(Copy, Clone)]
pub struct GroupEntry {
    // Note: Make sure to update the Debug impl below when adding a field here.
    read_format: ReadFormat,
    value: u64,
    id: u64,
    lost: u64,
}

impl GroupEntry {
    fn new(read_format: ReadFormat, data: &[u64]) -> Self {
        Self {
            read_format,
            value: data[0],
            id: data[1],
            lost: data.get(2).copied().unwrap_or(0),
        }
    }

    /// TODO
    pub fn value(&self) -> u64 {
        self.value
    }

    /// TODO
    pub fn id(&self) -> u64 {
        self.id
    }

    /// TODO
    pub fn lost(&self) -> Option<u64> {
        self.read_format
            .contains(ReadFormat::LOST)
            .then_some(self.lost)
    }
}

impl fmt::Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("GroupEntry");
        dbg.field("value", &self.value());
        dbg.field("id", &self.id());

        if let Some(lost) = self.lost() {
            dbg.field("lost", &lost);
        }

        dbg.finish_non_exhaustive()
    }
}

/// Iterator over the entries contained within [`GroupData`].
#[derive(Clone)]
pub struct GroupIter<'a> {
    read_format: ReadFormat,
    iter: std::slice::ChunksExact<'a, u64>,
}

impl<'a> GroupIter<'a> {
    fn new(read_format: ReadFormat, data: &'a [u64]) -> Self {
        Self {
            read_format,
            iter: data.chunks_exact(read_format.element_len()),
        }
    }
}

impl<'a> Iterator for GroupIter<'a> {
    type Item = GroupEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|chunk| GroupEntry::new(self.read_format, chunk))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter
            .nth(n)
            .map(|chunk| GroupEntry::new(self.read_format, chunk))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a> DoubleEndedIterator for GroupIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|chunk| GroupEntry::new(self.read_format, chunk))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter
            .nth_back(n)
            .map(|chunk| GroupEntry::new(self.read_format, chunk))
    }
}

impl<'a> ExactSizeIterator for GroupIter<'a> {}
impl<'a> FusedIterator for GroupIter<'a> {}
