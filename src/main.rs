use snafu::prelude::*;

// Must use this instead of std::error::Error to support no_std.
//
// This is a reexport of std::error::Error on std, and on no_std it is
// a trait very similar to std error, and also almost identical to the
// traits in the core_error and core2 crates.
pub use snafu::Error as CoreishError;

mod dep;
use dep::*;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use ::alloc::boxed::Box;

pub struct ErrorContainer {
    source: Box<dyn CoreishError + Send + Sync + 'static>,
}

impl core::fmt::Debug for ErrorContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.source.fmt(f)
    }
}

impl core::fmt::Display for ErrorContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.source.fmt(f)
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    #[snafu(display("Generic error: {source}"))]
    Generic {
        // XXX I wish I didn't have to do this
        source: ErrorContainer,
    },

    #[snafu(display("Specific error: {message}, from: {source}"))]
    Specific {
        message: String,
        source: OwnError,
    },

    #[snafu(display("Dependency error: {message}, from: {source}"))]
    DepGeneric {
        message: String,
        source: ErrorContainer,
    },

    #[cfg(feature = "std")]
    #[snafu(display("Dependency error: {message}, from: {source}"))]
    DepIO {
        message: String,
        source: std::io::Error,
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

fn main() {

    let own_err = OwnError { message: "reported genericly".to_string() };
    let own_res:Result<(), OwnError> = Result::Err(own_err);
    let generic_err = own_res.map_err(|e| e.into()).context(test_error::Generic);
    println!("G: {}", generic_err.unwrap_err());

    // let own_err = OwnError { message: "reported via whatever".to_string() };
    // let own_res:Result<(), OwnError> = Result::Err(own_err);
    // let whatever_generic_err:Result<(), TestError> = own_res.whatever_context("whatever generic message");
    // println!("W: {}", whatever_generic_err.unwrap_err());

    let own_err = OwnError { message: "reported specifically".to_string() };
    let own_res:Result<(), OwnError> = Result::Err(own_err);
    let specific_err = own_res.context(test_error::Specific{message: "specific message" });
    println!("S: {}", specific_err.unwrap_err());

    let dep_res = call_dep();
    println!("D: {}", dep_res.unwrap_err());

}
