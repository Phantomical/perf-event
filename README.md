## perf-event: a Rust interface to Linux performance monitoring

![example workflow](https://img.shields.io/github/actions/workflow/status/phantomical/perf-event/cargo.yml?style=flat-square)

This repository holds the source code for the [`perf-event2`] and
[`perf-event-open-sys2`] crates, which provide access to performance monitoring
hardware and software on linux.

This repository is a fork of Jim Blandy's [`perf-event`] crate that has been
updated to include several new features:
- The bindings have been updated to Linux 6.0
- `perf-event2` supports many more event types
- `perf-event2` supports reading and parsing profiling samples emitted by the
  kernel.

For more details see the readmes within the respective crate directories:
- [`perf-event2`](perf-event)
- [`perf-event-open-sys2`](perf-event-open-sys)

On systems other than Linux the `perf-event2` crate will not compile but you
can use the data types exposed by the `perf-event-open-sys2` crate. This can
useful for code that needs to parse perf-related data produced on Linux or
Android systems. The syscall and ioctl functions will not be available.

[`perf-event`]: https://github.com/jimblandy/perf-event
[`perf-event2`]: https://crates.io/crates/perf-event2
[`perf-event-open-sys2`]: https://crates.io/crates/perf-event-open-sys2

### See Also
- The original [`perf-event`] crate is still usable depending on your use case.
- If you want to parse data emitted by `perf` or `perf_event_open` see the 
  [`perf-event-data`] crate.

[`perf-event-data`]: https://crates.io/crates/perf-event-data
