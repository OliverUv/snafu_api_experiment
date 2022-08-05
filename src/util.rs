#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(feature = "std")]
use std::borrow::Cow;

use core::fmt::{Debug, Formatter, Result as FmtResult};

use snafu::prelude::*;

/// This is a reexport of std::error::Error on std, and on no_std it is
/// a trait very similar to std error, and also almost identical to the
/// traits in the core_error and core2 crates. We currently inherit this
/// trait from the snafu library - this is required for some of our 
/// error handling to work on no_std. We may switch to re-exporting it
/// from core_error pending https://github.com/shepmaster/snafu/issues/184
pub use snafu::Error as Error;

/// An alias for `Box<dyn Error + Send + Sync + 'static>` where the Error
/// trait is std::error::Error on std, and snafu::Error on no_std
pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

/// Defines an `impl From<FromType> for ToType`, which wraps the from
/// error in a Box and casts it with the given context_selector.
///
/// # Example
///
/// ```ignore
/// # #[macro_use] extern crate trinity_error;
/// err_from_via_box!(FromType, ToType, context_selector); // Generates:
///
/// impl From<FromType> for ToType {
///     fn from(err: FromType) -> Self {
///         use ::snafu::into_error;
///         context_selector.into_error(Box::new(err) as _)
///     }
/// }
/// ```
#[macro_export]
macro_rules! err_from_via_box {

    {$from:ty, $to:ty, $context_selector:path} => {
        impl From<$from> for $to {
            fn from(err: $from) -> Self {
                use ::snafu::IntoError;
                $context_selector.into_error(Box::new(err) as _)
            }
        }
    };

    {$from:ty, $to:ty, $context_selector:path,} => {
        $crate::err_from_via_box!($from, $to, $context_selector);
    };

}

pub trait BoxErrExt<O, E> {
    fn box_err(self) -> Result<O, BoxedError>;
}

impl<O, E: Error + Send + Sync + 'static> BoxErrExt<O, E> for Result<O, E> {
    fn box_err(self) -> Result<O, BoxedError> {
        self.map_err(|err| Box::new(err) as _)
    }
}

pub trait OptBoxErrExt<O, E> {
    fn opt_box_err(self) -> Result<O, OptionalBoxedError>;
}

impl<O, E: Error + Send + Sync + 'static> OptBoxErrExt<O, E> for Result<O, E> {
    fn opt_box_err(self) -> Result<O, OptionalBoxedError> {
        self.map_err(|err| OptionalBoxedError::new(Box::new(err) as _))
    }
}

// // Works but often requires type annotations.
// // so it's easier to do `.map_err(|err| Box::from(err) as _)` in the cases
// // where our error type isn't necessarily correct instead.
// impl<O, E, I> BoxErrExt<O, E> for Result<O, I> 
// where
//     E: Error + Send + Sync + 'static,
//     I: Into<E> + Send + Sync + 'static,
// {
//     fn box_err(self) -> Result<O, BoxedError> {
//         self.map_err(|err| Box::new(err.into()) as BoxedError)
//     }
// }

type Message = Cow<'static, str>;

#[derive(Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(display("{message}"))]
pub struct StringError {
    message: Message,
}

impl StringError {
    pub fn new(message: impl Into<Message>) -> Self {
        StringError {
            message: message.into(),
        }
    }
}

impl Debug for StringError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Debug::fmt(&self.message, f)
    }
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! str_err_fmt_impl {
    ($($arg:tt)*) => {{
        let err_str = format!($($arg)*);
        StringError::new(err_str)
    }}
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! str_err_fmt_impl {
    ($($arg:tt)*) => {{
        let err_str = alloc::format!($($arg)*);
        StringError::new(err_str)
    }}
}

#[derive(Debug)]
pub struct OptionalMessage(Option<Message>);

impl OptionalMessage {
    pub fn none() -> Self {
        OptionalMessage(None)
    }
}

impl core::fmt::Display for OptionalMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.0.as_ref() {
            None => write!(f, ""),
            Some(msg) => core::fmt::Display::fmt(msg, f),
        }
    }
}

impl<T> From<T> for OptionalMessage
where T: Into<Cow<'static, str>> {
    fn from(msg: T) -> Self {
        OptionalMessage(Some(msg.into()))
    }
}

/// Creates a new `StringError`. The given arguments are passed
/// to the format! macro to construct the error string.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate trinity_error;
/// # use trinity_error::*;
/// let s:StringError = str_err_fmt!("Hello {}!", "World");
/// let err_str = s.to_string();
/// assert_eq!(err_str, "Hello World!".to_string());
/// ```
#[macro_export]
macro_rules! str_err_fmt {
    ($($arg:tt)*) => {{
        $crate::str_err_fmt_impl!($($arg)*)
    }}
}

/// An error that must never be instantiated or used.
///
/// This is an Empty Type, which is even less of a Thing than a ZST.
/// See: https://doc.rust-lang.org/nomicon/exotic-sizes.html#empty-types
///
/// This is a hack while we wait for:
/// - `!` aka Never, https://github.com/rust-lang/rust/issues/35121
/// - `exhaustive_patterns`, https://github.com/rust-lang/rust/issues/51085
pub enum NeverError {}

impl core::fmt::Display for NeverError {
    fn fmt(&self, _f: &mut Formatter) -> FmtResult {
        unreachable!("NeverError must never be instantiated or used")
    }
}

impl core::fmt::Debug for NeverError {
    fn fmt(&self, _f: &mut Formatter) -> FmtResult {
        unreachable!("NeverError must never be instantiated or used")
    }
}

impl Error for NeverError {
    fn description(&self) -> &str {
        unreachable!("NeverError must never be instantiated or used")
    }
    fn cause(&self) -> Option<&dyn Error> {
        unreachable!("NeverError must never be instantiated or used")
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        unreachable!("NeverError must never be instantiated or used")
    }
}

/// Transforms an Option<Result<T>> into an Option<T> by ?-ing
/// the result if the option is Some.
///
/// # Expansion
///
/// ```ignore
/// try_inner!(thing)
/// // =>
/// match thing {
///     None => None,
///     Some(v) => Some(v?),
/// }
/// ```
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate trinity_error;
/// # use trinity_error::*;
/// fn f() -> Result<(), i32> {
///     let outer: Option<Result<(), i32>> = Some(Err(-1));
///     let outer = try_inner!(outer);
///     Ok(())
/// }
/// let res = f();
/// assert!(matches!(res, Err(-1)));
/// ```
///
/// ```
/// # #[macro_use] extern crate trinity_error;
/// # use trinity_error::*;
/// fn f() -> Result<Option<i32>, ()> {
///     let outer: Option<Result<i32, ()>> = Some(Ok(0));
///     let outer = try_inner!(outer);
///     Ok(outer)
/// }
/// let res = f();
/// assert!(matches!(res, Ok(Some(0))));
/// ```
#[macro_export]
macro_rules! try_inner {
    ($opt:ident) => {
        match $opt {
            None => None,
            Some(v) => Some(v?),
        }
    };
}

#[macro_export]
macro_rules! err_opt_none {
    ($err:expr) => {{
        use ::snafu::IntoError;
        let non_err = $crate::OptionalBoxedError::default();
        $err.into_error(non_err)
    }};

    ($err:path, $message:expr) => {{
        use ::snafu::IntoError;
        let non_err = $crate::OptionalBoxedError::default();
        $err{message: $message}.into_error(non_err)
    }};
}

#[derive(Debug)]
pub struct OptionalBoxedError(Option<BoxedError>);
impl Error for OptionalBoxedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.as_ref().map(|e| e.as_ref() as _)
    }

    // TODO: Backtrace? Probably impl snafu::ErrorCompat ?
    // fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
    //     self.0.as_ref().and_then(|e| e.backtrace())
    // }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl OptionalBoxedError {
    pub fn new(err: BoxedError) -> Self {
        Self(Some(err))
    }
}

impl Default for OptionalBoxedError {
    fn default() -> Self {
        OptionalBoxedError(None)
    }
}

impl core::fmt::Display for OptionalBoxedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => write!(f, "ERROR[P3_MT_0] Unknown cause"),
            Some(_) => write!(f, "ERROR[P3_MT_0] Caused by:"),
        }
    }
}

impl From<BoxedError> for OptionalBoxedError {
    fn from(err: BoxedError) -> Self {
        OptionalBoxedError(Some(err))
    }
}
