use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fmt, io};

use perf_event_open_sys::bindings::perf_event_attr;

use crate::events::Event;
use crate::events::x86::Msr;

used_in_docs!(Msr);

/// An event exposed as a dynamic PMU via the sysfs filesystem.
///
/// This type has no operations beyond implementing [`Event`]. Use
/// [`DynamicBuilder`] to build one.
#[derive(Copy, Clone, Debug)]
pub struct Dynamic {
    ty: u32,
    config: u64,
    config1: u64,
    config2: u64,
}

impl Dynamic {
    /// Construct a new dynamic builder for the provided perf PMU.
    ///
    /// See [`DynamicBuilder::new`].
    pub fn builder(pmu: impl AsRef<Path>) -> io::Result<DynamicBuilder> {
        DynamicBuilder::new(pmu)
    }
}

impl Event for Dynamic {
    fn update_attrs(self, attr: &mut perf_event_attr) {
        attr.type_ = self.ty;
        attr.config = self.config;
        attr.config1 = self.config1;
        attr.config2 = self.config2;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum FieldDest {
    Config,
    Config1,
    Config2,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum FieldBits {
    /// Field is a single bit
    Bit(u32),

    /// Field covers a range of bits.
    ///
    /// Note that `hi` is inclusive.
    Range { lo: u32, hi: u32 },
}

impl FieldBits {
    pub fn mask(self) -> u64 {
        match self {
            Self::Bit(bit) => 1u64 << bit,
            Self::Range { lo, hi } => (u64::MAX >> (63 - hi)) & (u64::MAX << lo),
        }
    }

    pub fn shift(self) -> u32 {
        match self {
            Self::Bit(bit) => bit,
            Self::Range { lo, .. } => lo,
        }
    }

    pub fn validate(&self, value: u64) -> bool {
        let mask = self.mask() >> self.shift();
        value & !mask == 0
    }
}

#[derive(Copy, Clone, Debug)]
struct Field {
    dest: FieldDest,
    bits: FieldBits,
    value: Option<u64>,
}

/// Builder for a [`Dynamic`] event.
///
/// The linux kernel exposes dynamic perfomance monitoring units (PMUs) using a
/// special filesystem under `/sys/bus/event_source/devices`. This builder reads
/// the config files for a specified PMU and allows you to set their values by
/// using the kernel-specified names instead of having to directly set the right
/// bits in the config fields.
///
/// If you find yourself having to use `Dynamic` and `DynamicBuilder` for an
/// event please consider submitting a PR to add a native [`Event`] impl for it
/// to this crate.
///
/// # Fields and Parameters
/// Generally, kernel PMUs expose a few pieces of info:
/// - A format, which defines fields and indicates which bits they correspond to
///   within [`perf_event_attr`].
/// - A set of events, which contain values to assign to the the fields defined
///   in the format.
///
/// Generally, all you need to do is to pick a PMU and an event within that PMU.
/// Here's how one would configure the [`Msr`] PMU using [`Dynamic`]:
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # use std::path::Path;
/// #
/// # match () {
/// #   _ if !cfg!(any(target_arch = "x86_64", target_arch = "i686")) => return Ok(()),
/// #   _ if !Path::new("/sys/bus/event_source/devices/msr").exists() => return Ok(()),
/// #   _ => ()
/// # }
/// #
/// use perf_event::events::Dynamic;
///
/// let event = Dynamic::builder("msr")?.event("tsc")?.build()?;
/// # Ok(())
/// # }
/// ```
///
/// You can use `perf list` to get a list of which kernel PMU events are
/// supported on the current machine. These will generally be listed in the
/// format `<pmu>/<event>/`. Here is a sample of some of the events on my
/// machine:
///
/// ```text
/// msr/aperf/                                         [Kernel PMU event]
/// msr/cpu_thermal_margin/                            [Kernel PMU event]
/// msr/mperf/                                         [Kernel PMU event]
/// msr/pperf/                                         [Kernel PMU event]
/// msr/smi/                                           [Kernel PMU event]
/// msr/tsc/                                           [Kernel PMU event]
/// power/energy-cores/                                [Kernel PMU event]
/// power/energy-gpu/                                  [Kernel PMU event]
/// power/energy-pkg/                                  [Kernel PMU event]
/// power/energy-psys/                                 [Kernel PMU event]
/// power/energy-ram/                                  [Kernel PMU event]
/// ```
///
/// In most cases, the event file has values for all the fields contained in the
/// format. However, for some events, the kernel leaves the field as an event
/// parameter which must be set by the user. In that case, you must set the
/// value for the field using the [`field`] method.
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # let pmu = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/dyn-pmu"));
/// use perf_event::events::Dynamic;
///
/// let event = Dynamic::builder(&pmu)?
///     .event("evt1")?
///     .field("param", 32)?
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// In cases where the kernel does not provide the desired event you can still
/// set the fields directly. Whether this works properly will ultimately depend
/// on the PMU itself. You will need to set all the fields or else [`build`]
/// will return an error.
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # let pmu = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/dyn-pmu"));
/// use perf_event::events::Dynamic;
///
/// let event = Dynamic::builder(&pmu)?
///     .field("event", 77)?
///     .field("param", 32)?
///     .field("flag", 1)?
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// [`build`]: Self::build
/// [`field`]: Self::field
#[derive(Clone)]
pub struct DynamicBuilder {
    ty: u32,
    pmu: PathBuf,
    event: Option<PathBuf>,
    fields: HashMap<String, Field>,
}

impl DynamicBuilder {
    /// Construct a new dynamic builder for the provided perf PMU.
    ///
    /// `pmu` can be either
    /// - an absolute path to the PMU directory, or,
    /// - the name of a pmu under `/sys/bus/event_source/devices`.
    ///
    /// This method will read the dynamic format configuration out of the PMU
    /// directory tree and configure the builder to use it.
    ///
    /// # Errors
    /// This method reads the contents of the format directory. Any IO errors
    /// from this will be returned directly. Any errors from parsing will
    /// have kind [`io::ErrorKind::Other`] with the inner error being a
    /// [`DynamicBuilderError`].
    pub fn new(pmu: impl AsRef<Path>) -> io::Result<Self> {
        Self::_new(pmu.as_ref())
    }

    fn _new(pmu: &Path) -> io::Result<Self> {
        let mut path = Path::new("/sys/bus/event_source/devices").to_path_buf();
        path.push(pmu);

        path.push("type");
        let ty: u32 = match std::fs::read_to_string(&path)?.trim().parse() {
            Ok(ty) => ty,
            Err(e) => return Err(Error::parse(path, e))?,
        };

        path.pop();
        path.push("format");

        let mut fields = HashMap::new();

        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let contents = std::fs::read_to_string(entry.path())?;

            let (name, rest) = contents
                .split_once(':')
                .ok_or_else(|| Error::missing_colon(entry.path()))?;

            let dest = match name {
                "config" => FieldDest::Config,
                "config1" => FieldDest::Config1,
                "config2" => FieldDest::Config2,
                _ => return Err(Error::unknown_target(entry.path(), name.to_owned()))?,
            };

            let bits = if let Some((first, rest)) = rest.split_once('-') {
                let lo: u32 = first.parse().map_err(|e| Error::parse(entry.path(), e))?;
                let hi: u32 = rest
                    .trim_end()
                    .parse()
                    .map_err(|e| Error::parse(entry.path(), e))?;

                FieldBits::Range { lo, hi }
            } else {
                let index = rest
                    .trim_end()
                    .parse()
                    .map_err(|e| Error::parse(entry.path(), e))?;

                FieldBits::Bit(index)
            };

            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| Error::invalid_field_name(entry.path()))?;

            fields.insert(
                name.to_owned(),
                Field {
                    dest,
                    bits,
                    value: None,
                },
            );
        }

        path.pop();

        Ok(Self {
            ty,
            pmu: path,
            event: None,
            fields,
        })
    }

    /// Initialize the builder for the specified event.
    ///
    /// `event` can be either
    /// - an absolute path to the event config file, or,
    /// - the name of an event file under
    ///   `/sys/bus/event_source/devices/<pmu>/events`.
    ///
    /// # Errors
    /// This method reads the contents of the event file. Any IO errors from
    /// this will be returned directly. Any errors from parsing will have kind
    /// [`io::ErrorKind::Other`] with the inner error being a
    /// [`DynamicBuilderError`].
    pub fn event(&mut self, event: impl AsRef<Path>) -> io::Result<&mut Self> {
        self._event(event.as_ref())
    }

    fn _event(&mut self, event: &Path) -> io::Result<&mut Self> {
        // The event file format is described here:
        // https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-bus-event_source-devices-events

        let mut path = self.pmu.join("events");
        path.push(event);

        let text = std::fs::read_to_string(&path)?;

        for term in text.split(',') {
            let term = term.trim_end();

            let (term, value) = match term.split_once('=') {
                Some((term, "?")) => (term, None),
                Some((term, value)) => match parse_hex(value) {
                    Ok(value) => (term, Some(value)),
                    Err(e) => return Err(Error::parse(path, e))?,
                },
                None => (term, Some(1u64)),
            };

            let field = match self.fields.get_mut(term) {
                Some(field) => field,
                None => return Err(Error::unknown_field(path, term.to_owned()))?,
            };

            field.value = value;
        }

        self.event = Some(path);
        Ok(self)
    }

    /// Set the value of a field.
    ///
    /// This overwrites the previous value of the field, if there was one.
    ///
    /// # Errors
    /// An error will be returned if
    /// - `field` did not exist in the format description for the PMU, or,
    /// - `value` was larger than the availabel bits for `field`.
    pub fn field(&mut self, field: &str, value: u64) -> Result<&mut Self, DynamicBuilderError> {
        let field = self
            .fields
            .get_mut(field)
            .ok_or_else(|| Error::new(ErrorData::UnknownField(field.to_owned())))?;

        if !field.bits.validate(value) {
            return Err(Error::new(ErrorData::ValueTooLarge));
        }

        field.value = Some(value);
        Ok(self)
    }

    /// Build the [`Dynamic`] event type using the fields in this builder.
    ///
    /// This will return an error if any event parameters have not been set to a
    /// value.
    pub fn build(&self) -> Result<Dynamic, MissingParameterError> {
        let mut dynamic = Dynamic {
            ty: self.ty,
            config: 0,
            config1: 0,
            config2: 0,
        };

        for (name, field) in self.fields.iter() {
            let target = match field.dest {
                FieldDest::Config => &mut dynamic.config,
                FieldDest::Config1 => &mut dynamic.config1,
                FieldDest::Config2 => &mut dynamic.config2,
            };

            let mask = field.bits.mask();
            let value = match field.value {
                Some(value) => (value << field.bits.shift()) & mask,
                None => return Err(MissingParameterError::new(name.to_owned())),
            };

            *target &= !mask;
            *target |= value;
        }

        Ok(dynamic)
    }

    /// Iterate over all unset parameter fields.
    ///
    /// By default, no fields are parameter fields. They only become parameter
    /// fields when the event file contains `<field>=?`.
    pub fn params(&self) -> impl Iterator<Item = &str> {
        self.fields()
            .filter(|(_, value)| value.is_none())
            .map(|(key, _)| key)
    }

    /// Iterate over all fields in this builder.
    pub fn fields(&self) -> impl Iterator<Item = (&str, Option<u64>)> {
        self.fields.iter().map(|(key, field)| (&**key, field.value))
    }

    fn property(&self, suffix: &str) -> io::Result<(PathBuf, Option<String>)> {
        let event = match &self.event {
            Some(event) => event,
            None => return Err(Error::new(ErrorData::MissingEvent))?,
        };

        let path = event.with_extension(suffix);
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => Some(content),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };

        Ok((path, content))
    }

    /// Read the scale factor of the event.
    ///
    /// This is a value to be multiplied by the event count emitted by the
    /// kernel in order to convert the count to the unit as returned by
    /// [`unit`](Self::unit). Not all events have a scale.
    ///
    /// If [`event`](Self::event) has not been specified then this method will
    /// always return an error.
    pub fn scale(&self) -> io::Result<Option<f64>> {
        let (path, content) = match self.property("scale")? {
            (_, None) => return Ok(None),
            (path, Some(content)) => (path, content),
        };

        let scale: f64 = match content.trim().parse() {
            Ok(scale) => scale,
            Err(e) => return Err(Error::parse_float(path, e))?,
        };

        Ok(Some(scale))
    }

    /// Read the unit of the event.
    ///
    /// This is a string describing the english unit that the event represents
    /// (once multiplied by [`scale`](Self::scale)). Not all events have a unit.
    ///
    /// If [`event`](Self::event) has not been specified then this method will
    /// always return an error.
    pub fn unit(&self) -> io::Result<Option<String>> {
        Ok(match self.property("unit")? {
            (_, Some(mut content)) => {
                let trimmed = content.trim_end();
                content.truncate(trimmed.len());
                Some(content)
            }
            (_, None) => None,
        })
    }
}

fn parse_hex(text: &str) -> Result<u64, std::num::ParseIntError> {
    let text = text.strip_prefix("0x").unwrap_or(text);
    u64::from_str_radix(text, 16)
}

impl fmt::Debug for DynamicBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("DynamicBuilder");
        dbg.field("type", &self.ty);
        dbg.field("pmu", &self.pmu);

        if let Some(event) = &self.event {
            dbg.field("event", &event);
        }

        dbg.field("fields", &DebugFields(self));
        dbg.finish()
    }
}

struct DebugValue(Option<u64>);

impl fmt::Debug for DebugValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(value) => value.fmt(f),
            None => f.write_str("?"),
        }
    }
}

struct DebugFields<'a>(&'a DynamicBuilder);

impl fmt::Debug for DebugFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(
                self.0
                    .fields()
                    .map(|(name, value)| (name, DebugValue(value))),
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
enum ErrorData {
    /// The destination field for the config was not one of the ones we support.
    UnknownTarget(String),

    /// The field name was not one of the ones in the PMU format description.
    UnknownField(String),

    /// The value was larger than the maximum value supported by the field.
    ValueTooLarge,

    /// We were unable to parse one of the integers within the config file.
    InvalidInteger(std::num::ParseIntError),

    // Same thing, but a float instead
    InvalidFloat(std::num::ParseFloatError),

    /// The field name was not valid UTF-8
    NonUtf8FieldName,

    /// When reading a format file we were unable to find the colon separating
    /// the field target from its bit specification.
    MissingColon,

    /// We tried to read an event property but no event has been specified for
    /// the builder.
    MissingEvent,

    /// A required parameter was specified in the format but it was not provided
    /// by the user.
    MissingParam(String),
}

type Error = DynamicBuilderError;

/// Error for when the PMU config files are invalid.
#[derive(Debug)]
pub struct DynamicBuilderError {
    data: ErrorData,
    path: Option<PathBuf>,
}

impl DynamicBuilderError {
    pub(self) fn new(data: ErrorData) -> Self {
        Self { data, path: None }
    }

    fn missing_colon(path: PathBuf) -> Self {
        Self {
            data: ErrorData::MissingColon,
            path: Some(path),
        }
    }

    fn parse(path: PathBuf, e: std::num::ParseIntError) -> Self {
        Self {
            data: ErrorData::InvalidInteger(e),
            path: Some(path),
        }
    }

    fn parse_float(path: PathBuf, e: std::num::ParseFloatError) -> Self {
        Self {
            data: ErrorData::InvalidFloat(e),
            path: Some(path),
        }
    }

    fn invalid_field_name(path: PathBuf) -> Self {
        Self {
            data: ErrorData::NonUtf8FieldName,
            path: Some(path),
        }
    }

    fn unknown_target(path: PathBuf, target: String) -> Self {
        Self {
            data: ErrorData::UnknownTarget(target),
            path: Some(path),
        }
    }

    fn unknown_field(path: PathBuf, field: String) -> Self {
        Self {
            data: ErrorData::UnknownField(field),
            path: Some(path),
        }
    }
}

impl fmt::Display for ErrorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTarget(field) => write!(f, "unknown target field `{field}`"),
            Self::UnknownField(field) => write!(f, "unknown field `{field}`"),
            Self::ValueTooLarge => write!(f, "value was too large for the field"),
            Self::InvalidInteger(e) => e.fmt(f),
            Self::InvalidFloat(e) => e.fmt(f),
            Self::NonUtf8FieldName => write!(f, "field name contained invalid UTF-8"),
            Self::MissingColon => write!(f, "expected a ':', found EOF instead"),
            Self::MissingEvent => write!(f, "need a configured event to read an event property"),
            Self::MissingParam(param) => write!(f, "missing required parameter `{param}`"),
        }
    }
}

impl fmt::Display for DynamicBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "invalid PMU config file `{}`", path.display()),
            None => self.data.fmt(f),
        }
    }
}

impl std::error::Error for ErrorData {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInteger(e) => Some(e),
            _ => None,
        }
    }
}

impl std::error::Error for DynamicBuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.path {
            Some(_) => Some(&self.data),
            None => self.data.source(),
        }
    }
}

/// A required PMU parameter did not have a value.
///
/// Some dynamic events have event parameters. These need to be set to a value
/// before a dynamic event can be built. If they are not, then
/// [`DynamicBuilder::build`] will emit this error.
#[derive(Clone, Debug)]
pub struct MissingParameterError {
    param: String,
}

impl MissingParameterError {
    fn new(param: String) -> Self {
        Self { param }
    }

    /// The name of the parameter that was missing.
    pub fn param(&self) -> &str {
        &self.param
    }
}

impl fmt::Display for MissingParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing required parameter `{}`", self.param)
    }
}

impl std::error::Error for MissingParameterError {}

impl From<DynamicBuilderError> for io::Error {
    fn from(value: DynamicBuilderError) -> Self {
        io::Error::other(value)
    }
}

impl From<MissingParameterError> for DynamicBuilderError {
    fn from(value: MissingParameterError) -> Self {
        Self::new(ErrorData::MissingParam(value.param))
    }
}

impl From<MissingParameterError> for io::Error {
    fn from(value: MissingParameterError) -> Self {
        io::Error::other(value)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

    fn pmu_enabled(pmu: &str) -> bool {
        let path = Path::new("/sys/bus/event_source/devices").join(pmu);
        path.exists()
    }

    fn test_pmu_dir() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data"))
    }

    #[test]
    fn parse_hex_sanity() {
        assert_eq!(parse_hex("0xFFFFFFFF"), Ok(0xFFFFFFFF));
    }

    #[test]
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    fn dynamic_msr_event() {
        if !pmu_enabled("msr") {
            return;
        }

        let _event = Dynamic::builder("msr")
            .unwrap()
            .event("tsc")
            .unwrap()
            .build()
            .unwrap();
    }

    #[test]
    fn dynamic_pmu_evt1() {
        let pmu = test_pmu_dir().join("dyn-pmu");
        let mut builder = Dynamic::builder(pmu).unwrap();
        builder.event("evt1").unwrap();

        let fields: HashMap<_, _> = HashMap::from_iter(builder.fields());
        assert_eq!(fields["param"], None);
        assert_eq!(fields["flag"], Some(1));
        assert_eq!(fields["event"], Some(0xFFFFFFFF));

        assert_eq!(builder.scale().unwrap(), Some(0.5));
        assert_eq!(builder.unit().unwrap().as_deref(), Some("Frogs"));

        builder
            .build()
            .expect_err("param not set, build should have errored");
        builder.field("param", 0x770).unwrap();
        let event = builder.build().unwrap();

        assert_eq!(event.ty, 66666);
        assert_eq!(event.config, 0x770FFFFFFFF);
        assert_eq!(event.config1, 0x1);
        assert_eq!(event.config2, 0);
    }

    #[test]
    fn dynamic_pmu_evt2() {
        let pmu = test_pmu_dir().join("dyn-pmu");
        let mut builder = Dynamic::builder(pmu).unwrap();
        builder.event("evt2").unwrap();

        let fields: HashMap<_, _> = HashMap::from_iter(builder.fields());
        assert_eq!(fields["param"], Some(0x45));
        assert_eq!(fields["flag"], Some(0x00));
        assert_eq!(fields["event"], Some(0xABCDEF));

        assert_eq!(builder.scale().unwrap(), None);
        assert_eq!(builder.unit().unwrap().as_deref(), None);

        let event = builder.build().unwrap();
        assert_eq!(event.ty, 66666);
        assert_eq!(event.config, 0x4500ABCDEF);
        assert_eq!(event.config1, 0);
        assert_eq!(event.config2, 0);
    }

    #[test]
    fn dynamic_pmu_empty() {
        let pmu = test_pmu_dir().join("dyn-pmu");
        let builder = Dynamic::builder(pmu).unwrap();

        let fields: HashMap<_, _> = HashMap::from_iter(builder.fields());
        assert_eq!(fields["param"], None);
        assert_eq!(fields["flag"], None);
        assert_eq!(fields["event"], None);
    }
}
