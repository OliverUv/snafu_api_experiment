use snafu::prelude::*;
use snafu::Error as CoreishError;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use ::alloc::boxed::Box;

// XXX Getting an LSP error saying test_error is an unresolved import.
use crate::{ErrorContainer, TestError, test_error};

#[derive(Debug, Snafu)]
enum DependencyError {

    #[snafu(display("could not reticulate: {item}"))]
    Reticulate {
        item: String,
    },

    #[cfg(feature = "std")]
    #[snafu(display("io error: {message}, from: {source}"))]
    IO {
        message: String,
        source: std::io::Error,
    },
}

impl Into<TestError> for DependencyError {
    fn into(self) -> TestError {
        use DependencyError as DE;
        match self {

            DE::Reticulate{..} => {
                let error_container = ErrorContainer {
                    source: Box::new(self) as _,
                };
                // XXX I wish there was a better way to do this
                let r = Err(error_container);
                let err:Result<(), TestError> = r.context(test_error::Generic);
                err.unwrap_err()
            },

            #[cfg(feature = "std")]
            DE::IO{message, source} => {
                let r = Err(source);
                let err:Result<(), TestError> = r.context(test_error::DepIO{ message });
                err.unwrap_err()
            },

        }
    }
}

pub fn call_dep() -> Result<(), TestError> {
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
    ensure!(false, ReticulateSnafu{ item: "splines" });
    Ok(())
}
