/// Macro for defining a binding to a C-like enum.
///
/// Normally, we would like to use a rust enum to represent a C enum. However,
/// with an interface like perf_event_open most of the enums we are dealing
/// with can gain new variants in a backwards compatible manner. If we tried
/// to use rust enums for this we'd end up with messy conversions and an
/// awkward `Unknown(x)` variant on every enum. In addition, adding a new
/// variant would break downstream code that was relying on `Unknown(x)`
/// working.
///
/// The solution to this is to not use rust enums. Instead, we define a C enum
/// wrapper struct like this
/// ```
/// pub struct MyEnum(pub u32);
/// ```
/// and then add associated constants for all the enum variants.
///
/// This macro is a helper macro (in the style of bitflags!) which defines an
/// enum as described above and also derives a specialized Debug impl for it.
///
/// # Example
/// If we declare a simple enum like this
/// ```ignore
/// c_enum! {
///     /// Insert docs here
///     pub struct SomeEnum : u32 {
///         const A = 0;
///         const B = 1;
///     }
/// }
/// ```
///
/// Then the resulting rust code would look (roughly) like this
/// ```
/// /// Insert docs here
/// #[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
/// pub struct SomeEnum(pub u32);
///
/// #[allow(missing_docs)]
/// impl SomeEnum {
///     pub const A: Self = Self(0);
///     pub const B: Self = Self(1);
/// }
///
/// impl std::fmt::Debug for SomeEnum {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             &Self::A => f.write_str("SomeEnum::A"),
///             &Self::B => f.write_str("SomeEnum::B"),
///             Self(value) => f.debug_tuple("SomeEnum").field(value).finish(),
///         }
///     }
/// }
///
/// impl std::convert::From<u32> for SomeEnum {
///     fn from(value: u32) -> Self {
///         Self(value)
///     }
/// }
/// ```
macro_rules! c_enum {
    {
        $( #[doc = $doc:expr] )*
        $( #[allow($warning:ident)] )*
        $vis:vis struct $name:ident : $inner:ty {
            $(
                $( #[ $field_attr:meta ] )*
                const $field:ident = $value:expr;
            )*
        }
    } => {
        $( #[doc = $doc] )*
        $( #[allow($warning)] )*
        #[derive(Copy, Clone, Eq, PartialEq, Hash)]
        $vis struct $name(pub $inner);

        $( #[allow($warning)] )*
        impl $name {
            $(
                $( #[$field_attr] )*
                pub const $field: Self = Self($value);
            )*
        }

        impl $name {
            #[doc = concat!("Create a new `", stringify!($name), "` from a `", stringify!($inner), "`.")]
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match *self {
                    $( Self::$field => f.write_str(concat!(stringify!($name), "::", stringify!($field))), )*
                    Self(value) => f.debug_tuple(stringify!($name)).field(&value).finish()
                }
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
    }
}
