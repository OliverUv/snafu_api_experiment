use snafu::prelude::*;

#[cfg(not(feature = "std"))]
use ::alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use ::alloc::string::String;

// XXX Getting an LSP error saying test_error is an unresolved import.
use crate::{TestError, test_error, BoxedErr};

#[derive(Debug, Snafu)]
enum DependencyError {

    #[snafu(display("ERROR[P3_SN_DEP_1] could not reticulate: {item}"))]
    Reticulate {
        item: String,
        // Capture a backtrace
        backtrace: snafu::Backtrace,
    },

    #[cfg(feature = "std")]
    #[snafu(display("ERROR[P3_SN_DEP_2] io error: {message}, from: {source}"))]
    IO {
        message: String,
        source: std::io::Error,
        // Capture a backtrace since io::Error does not
        backtrace: snafu::Backtrace,
    },
}

impl Into<TestError> for DependencyError {
    fn into(self) -> TestError {
        use DependencyError as DE;
        match self {

            // XXX I wish there was a better way to do
            // this. We do not manually construct and return a 
            // TestError::Generic{source: error_container, backtrace: None}
            // since this interferes with Snafu's backtrace generating
            // & passing machinery.

            DE::Reticulate{..} => {
                // Either the `let r` or `let err` lines must be given
                // a type annotation
                let r:Result<(), BoxedErr> = Err(Box::new(self) as _);
                let err = r.context(test_error::DepGeneric);
                err.unwrap_err()
            },

            #[cfg(feature = "std")]
            DE::IO{message, source, ..} => {
                let r = Err(source);
                let err:Result<(), TestError> =
                    r.context(test_error::DepIO{ message });
                err.unwrap_err()
            },

        }
    }
}

pub fn call_dep() -> Result<(), TestError> {
    // We could have the dependencies just return their own
    // Error types and call `.map_err(|e| Box::new(e) as _)`
    // on them, but then they would all turn into generic errors
    // which I don't think we want.
    inner().map_err(|e| e.into())
}

#[cfg(feature = "std")]
fn inner() -> Result<(), DependencyError> {
    let file = "/ASDASDFASDASDFASDF";
    let _ = std::fs::read_to_string(file)
        .context(IOSnafu{message: format!("could not read from file {}", file)})?;
    Ok(())
}

#[cfg(not(feature = "std"))]
fn inner() -> Result<(), DependencyError> {
    return Err(ReticulateSnafu{ item: "splines" }.build());
}
