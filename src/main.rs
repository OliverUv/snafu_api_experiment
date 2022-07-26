#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(default_alloc_error_handler))]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use ::alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use ::alloc::string::String;
#[cfg(not(feature = "std"))]
use ::alloc::string::ToString;

// Must use this instead of std::error::Error to support no_std.
//
// This is a reexport of std::error::Error on std, and on no_std it is
// a trait very similar to std error, and also almost identical to the
// traits in the core_error and core2 crates.
pub use snafu::Error as CoreishError;
use snafu::prelude::*;

use libc_print::libc_println;

mod dep;
use dep::call_dep;

pub type BoxedErr = Box<dyn CoreishError + Send + Sync + 'static>;

#[derive(Debug)]
pub struct NonSnafuError {
    pub message: String,
}

impl ::core::fmt::Display for NonSnafuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Snafu requires this specific trait to be implemented. Any home-rolled
// Core-ish Error traits do not work, neither does simply impl:ing
// Display and Debug.
impl CoreishError for NonSnafuError {}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(module)]
#[snafu(context(suffix(false)))]
pub enum TestError {
    #[snafu(display("ERROR[P3_SN_1] Generic error"))]
    Generic {
        source: BoxedErr,
        backtrace: Option<snafu::Backtrace>,
    },

    #[snafu(display("ERROR[P3_SN_2] Specific error: {message}"))]
    Specific {
        message: String,
        source: NonSnafuError,
        backtrace: Option<snafu::Backtrace>,
    },

    #[snafu(display("ERROR[P3_SN_3] Generic dependency error"))]
    DepGeneric {
        source: BoxedErr,
        backtrace: Option<snafu::Backtrace>,
    },

    #[cfg(feature = "std")]
    #[snafu(display("ERROR[P3_SN_4] Dependency IO error: {message}"))]
    DepIO {
        message: String,
        source: std::io::Error,
        backtrace: Option<snafu::Backtrace>,
    },
}

fn report_error<T>(err: Result<T, TestError>) {
    let err = match err {
        Ok(_) => return,
        Err(e) => e,
    };

    // Have submitted a PR to snafu to make this work on no_std
    // https://github.com/shepmaster/snafu/pull/343
    snafu::ErrorCompat::iter_chain(&err).for_each(|e| {
        libc_println!("{}", e);
    });

    let bt = snafu::ErrorCompat::backtrace(&err);
    if let Some(bt) = bt {
        libc_println!("BACKTRACE: {:?}", bt);
    }
}

fn main() {

    let non_s_err = NonSnafuError { message: "NonSnafuError reported genericly".to_string() };
    let non_s_res:Result<(), NonSnafuError> = Result::Err(non_s_err);
    let generic_err = non_s_res
        .map_err(|e| Box::new(e) as _)
        .context(test_error::Generic);
    libc_println!("\nGeneric:");
    report_error(generic_err);

    let non_s_err = NonSnafuError { message: "NonSnafuError reported specifically".to_string() };
    let non_s_res:Result<(), NonSnafuError> = Result::Err(non_s_err);
    let specific_err = non_s_res.context(test_error::Specific{message: "specific message" });
    libc_println!("\nSpecific:");
    report_error(specific_err);

    let dep_res = call_dep();
    libc_println!("\nError from dependency:");
    report_error(dep_res);

}
