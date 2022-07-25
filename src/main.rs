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

#[derive(Debug)]
pub struct ErrorContainer {
    source: Box<dyn CoreishError + Send + Sync + 'static>,
}

impl core::fmt::Display for ErrorContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "(ErrorContainer)")
    }
}

impl CoreishError for ErrorContainer {
    fn source(&self) -> Option<&(dyn CoreishError + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
pub struct OwnError {
    pub message: String,
}

impl ::core::fmt::Display for OwnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Snafu requires this specific trait to be implemented. Any home-rolled
// Core-ish Error traits do not work, neither does simply impl:ing
// Display and Debug
impl CoreishError for OwnError {}

impl Into<ErrorContainer> for OwnError {
    fn into(self) -> ErrorContainer {
        ErrorContainer {
            source: Box::new(self),
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(module)]
#[snafu(context(suffix(false)))]
pub enum TestError {
    #[snafu(display("Generic error"))]
    Generic {
        // XXX I wish I didn't have to do this
        source: ErrorContainer,
        backtrace: Option<snafu::Backtrace>,
    },

    #[snafu(display("Specific error: {message}"))]
    Specific {
        message: String,
        source: OwnError,
        backtrace: Option<snafu::Backtrace>,
    },

    #[snafu(display("Generic dependency error"))]
    DepGeneric {
        source: ErrorContainer,
        backtrace: Option<snafu::Backtrace>,
    },

    #[cfg(feature = "std")]
    #[snafu(display("Dependency IO error: {message}"))]
    DepIO {
        message: String,
        source: std::io::Error,
        backtrace: Option<snafu::Backtrace>,
    },

    // // XXX Does not work in no_std, FromString in Snafu is only
    // // impl'd in std. This is not very high on my list of features
    // // I want though, because I don't enjoy the `whatever!` api.
    //
    // #[snafu(whatever, display("Whatever-generic error: {message}"))]
    // WhateverGeneric {
    //     message: String,
    //     #[snafu(source(from(Box<dyn CoreishError + Send + Sync + 'static>, Some)))]
    //     source: Option<Box<dyn CoreishError + Send + Sync + 'static>>,
    // },

}

fn report_error<T>(err: Result<T, TestError>) {
    let err = match err {
        Ok(_) => return,
        Err(e) => e,
    };

    snafu::ErrorCompat::iter_chain(&err).for_each(|e| {
        libc_println!("{}", e);
    });

    let _ = {
        let bt = snafu::ErrorCompat::backtrace(&err);
        if let Some(bt) = bt {
            libc_println!("BACKTRACE: {:?}", bt);
        }
    };
}

fn main() {

    let own_err = OwnError { message: "OwnError reported genericly".to_string() };
    let own_res:Result<(), OwnError> = Result::Err(own_err);
    let generic_err = own_res
        .map_err(|e| e.into())
        .context(test_error::Generic);
    libc_println!("\nGeneric:");
    report_error(generic_err);

    // let own_err = OwnError { message: "reported via whatever".to_string() };
    // let own_res:Result<(), OwnError> = Result::Err(own_err);
    // let whatever_generic_err:Result<(), TestError> = own_res.whatever_context("whatever generic message");
    // libc_println!("W: {}", whatever_generic_err.unwrap_err());

    let own_err = OwnError { message: "OwnError reported specifically".to_string() };
    let own_res:Result<(), OwnError> = Result::Err(own_err);
    let specific_err = own_res.context(test_error::Specific{message: "specific message" });
    libc_println!("\nSpecific:");
    report_error(specific_err);

    let dep_res = call_dep();
    libc_println!("\nDependent:");
    report_error(dep_res);

}
