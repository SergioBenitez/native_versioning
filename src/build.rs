extern crate cc;

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Error enum.
#[derive(Debug)]
pub enum Error {
    Io(::std::io::Error),
    EnvVar(::std::env::VarError),
    Fmt(::std::fmt::Error)
}

impl From<::std::io::Error> for Error {
    fn from(error: ::std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<::std::env::VarError> for Error {
    fn from(error: ::std::env::VarError) -> Self {
        Error::EnvVar(error)
    }
}

impl From<::std::fmt::Error> for Error {
    fn from(error: ::std::fmt::Error) -> Self {
        Error::Fmt(error)
    }
}

#[macro_export]
fn version() -> Result<String, Error> {
    use std::fmt::Write;

    let mut version = String::new();
    write!(version, "_v{}", env::var("CARGO_PKG_VERSION_MAJOR")?)?;
    write!(version, "_{}", env::var("CARGO_PKG_VERSION_MINOR")?)?;
    write!(version, "_{}", env::var("CARGO_PKG_VERSION_PATCH")?)?;
    write!(version, "_{}", env::var("CARGO_PKG_VERSION_PRE")?)?;
    Ok(version)
}

/// Trait that provides the `include_header()` method for `cc::Build`.
pub trait HeaderInclude {
    /// Adds a header file to this compilation.
    ///
    /// Adding a header via this method has the same effect as specifying the
    /// file with double quotation marks in an `#include` directive on the first
    /// line of every source file being compiled by `self`. If you use this
    /// method multiple times, files are included in the order this method is
    /// called.
    ///
    /// XX: This doc is mostly copied from MSDN. That okay?
    ///
    /// # Panics
    ///
    /// Panics if an error occurred while determining the compiler that will be
    /// used or if the determined compiler is unknown.
    fn include_header<P: AsRef<Path>>(&mut self, header: P) -> &mut Self;
}

impl HeaderInclude for cc::Build {
    fn include_header<P: AsRef<Path>>(&mut self, header: P) -> &mut Self {
        let compiler = self.get_compiler();
        if compiler.is_like_gnu() || compiler.is_like_clang() {
            self.flag("-include").flag(&header.as_ref().display().to_string())
        } else if compiler.is_like_msvc() {
            self.flag("/FI").flag(&header.as_ref().display().to_string())
        } else {
            panic!("determined compiler is unknown")
        }
    }
}

/// Generates the versioned header file with the version mangling macro.
///
/// The header is generated in a file named `header_filename` in the path
/// `include_dir`. The versioned macro will be named `macro_name`.
pub fn write_versioned_header<I, H>(
    include_dir: I,
    header_filename: H,
    macro_name: &str
) -> Result<PathBuf, Error>
    where I: AsRef<Path>, H: AsRef<Path>
{
    let include_dir = include_dir.as_ref();
    let versioned_h = include_dir.join(header_filename.as_ref());

    fs::create_dir_all(include_dir)?;
    let mut file = File::create(&versioned_h)?;
    write!(file, "#define {}(sym) sym ## {}\n", macro_name, version()?)?;

    Ok(versioned_h)
}
