use snafu::prelude::*;

// Must use this instead of std::error::Error to support no_std.
//
// This is a reexport of std::error::Error on std, and on no_std it is
// a trait very similar to std error, and also almost identical to the
// traits in the core_error and core2 crates.
use snafu::Error as CoreishError;

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
struct DependencyError {
    pub message: String,
}

impl ::core::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Snafu requires this specific trait to be implemented. Any home-rolled
// Core-ish Error traits do not work, neither does simply impl:ing
// Display and Debug
impl CoreishError for DependencyError {}

impl Into<ErrorContainer> for DependencyError {
    fn into(self) -> ErrorContainer {
        ErrorContainer {
            source: Box::new(self),
        }
    }
}

#[derive(Debug, Snafu)]
enum TestError {
    #[snafu(display("Generic error: {message}, from: {source}"))]
    Generic {
        message: String,
        source: ErrorContainer,
    },

    // // DOES NOT WORK IN NO_STD, FromString in Snafu is only impl'd in std
    //
    // #[snafu(whatever, display("Whatever-generic error: {message}"))]
    // WhateverGeneric {
    //     message: String,
    //     #[snafu(source(from(Box<dyn CoreishError + Send + Sync + 'static>, Some)))]
    //     source: Option<Box<dyn CoreishError + Send + Sync + 'static>>,
    // },

    #[snafu(display("Specific error: {message}, from: {source}"))]
    Specific {
        message: String,
        source: DependencyError,
    },
}

fn main() {

    let dep_err = DependencyError { message: "reported genericly".to_string() };
    let dep_res:Result<(), DependencyError> = Result::Err(dep_err);
    let generic_err = dep_res.map_err(|e| e.into()).context(GenericSnafu{message: "generic message"});
    println!("G: {}", generic_err.unwrap_err());

    // let dep_err = DependencyError { message: "i broke in some dependency".to_string() };
    // let dep_res:Result<(), DependencyError> = Result::Err(dep_err);
    // let whatever_generic_err:Result<(), TestError> = dep_res.whatever_context("whatever generic message");
    // println!("W: {}", whatever_generic_err.unwrap_err());

    let dep_err = DependencyError { message: "reported specifically".to_string() };
    let dep_res:Result<(), DependencyError> = Result::Err(dep_err);
    let specific_err = dep_res.context(SpecificSnafu{message: "specific message" });
    println!("S: {}", specific_err.unwrap_err());

}
