# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.7.4 - 2023-05-30
### Added
- Added several new methods on `Builder` for some fields which were not
  previously wrapped.
- Added `Builder::event` method which allows reconfiguring the event that the
  builder was initially created with.

### Changed
- All manpage links in the docs now point to https://mankier.com

## 0.7.3 - 2023-05-26
### Added
- Add `Group::builder` method which creates a builder that is preconfigured for
  constructing a `Group`. @daniel-levin

## 0.7.2 - 2023-10-22
### Fixed
- Fixed a panic in `Sampler` when handling a record that wrapped around the end
  of the sampler ring buffer.

## 0.7.1 - 2023-10-05
### Added
- Added a `Raw` event type for creating `PERF_TYPE_RAW` counter events.
  @janaknat

## 0.7.0 - 2023-07-24
### Added
- Add `Sampler::read_user` method to for reading counters from userspace.
  Only x86 and x86_64 are supported when reading counters. 

### Changed
- **(breaking)** `Sampler::next_record` now takes `&mut self` instead of `&self`.
  This fixes UB that could arise due to having multiple `Record`s from the same
  `Sampler` existing at the same time.

## 0.6.3 - 2023-05-30
### Added
- Introduce a new `Dynamic` event type along with its builder. These allow
  easily creating counters for dynamic PMUs exposed via sysfs.

## 0.6.2 - 2023-05-26
### Added
- Introduce a new `x86::Msr` event type to expose the perf-event msr PMU. @yangxi

## 0.6.1 - 2023-05-19
### Added
- Expose the `IOC_SET_BPF` ioctl as `Counter::set_bpf`.
- Add `Event::update_attrs_with_data` to allow events to store references to
  owned data within `Builder`'s `perf_event_attr` struct.
- Add `KProbe`, `UProbe`, and `Tracepoint` event types.

## 0.6.0 - 2023-05-17
### Added
- Expose the `perf_event_data` crate as the `data` module.
- Add `Record::parse_record` to parse records to `data::Record`.
- Add `Software::CGROUP_SWITCHES` and `Software::BPF_OUTPUT` events (#9). @Phantomical

### Changed
- `Hardware` is no longer a rust enum. The constants remain the same.
- `Software` is no longer a rust enum. The constants remain the same.
- The same applies for `WhichCache`, `CacheOp`, and `CacheResult`.
- `WhichCache` has been renamed to `CacheId`.

## 0.5.0 - 2023-04-20
### Added
- Add `Sampler` - a `Counter` which also reads sample events emitted by the kernel.
- Group leaders can now be a `Group`, `Counter`, or `Sampler`.
- Add `Builder::build_group` to build a group with a non-default config.
- Add all missing config options for `Builder`.

### Changed
- The `Event` enum has been replaced with an `Event` trait.
- Constructing a `Builder` now requires that you specify an event type up front
  instead of having a default of `Hardware::INSTRUCTIONS`.