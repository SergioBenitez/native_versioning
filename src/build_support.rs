extern crate cc;

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const ENV_NAME: &str = "NATIVE_VERSIONING_VERSION";

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

fn git_shorthash() -> io::Result<Option<String>> {
    let git_base = Path::new(".git");
    if let Err(e) = fs::metadata(git_base) {
        match e.kind() {
            io::ErrorKind::NotFound => return Ok(None),
            _ => return Err(e)
        }
    }

    let mut contents = String::new();
    let mut file = File::open(git_base.join("HEAD"))?;
    file.read_to_string(&mut contents)?;

    if contents.starts_with("ref: ") {
        let ref_path = git_base.join(&contents[5..].trim_right());
        let mut ref_file = File::open(ref_path)?;
        contents.truncate(0);
        ref_file.read_to_string(&mut contents)?;
    }

    if contents.len() < 8 {
        Err(io::Error::new(io::ErrorKind::InvalidData, "invalid git ref"))
    } else {
        Ok(Some(contents[..8].into()))
    }
}

fn crate_version() -> Result<String, Error> {
    use std::fmt::Write;

    let mut version = String::new();
    write!(version, "v{}", env::var("CARGO_PKG_VERSION_MAJOR")?)?;
    write!(version, "_{}", env::var("CARGO_PKG_VERSION_MINOR")?)?;
    write!(version, "_{}", env::var("CARGO_PKG_VERSION_PATCH")?)?;

    let pre = env::var("CARGO_PKG_VERSION_PRE")?;
    if !pre.is_empty() {
        write!(version, "_{}", pre)?;
    }

    Ok(version)
}

fn version() -> Result<String, Error> {
    use std::fmt::Write;

    let mut version = crate_version()?;
    if let Some(shorthash) = git_shorthash()? {
        write!(version, "_{}", shorthash)?;
    }

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

/// Generates the versioned header file with the version mangling CPP macro and
/// exports an environment variable with the current project's version.
///
/// The header is generated in a file named `header_filename` in the path
/// `include_dir`. The versioned macro will be named `macro_name`. The
/// environment variable is exported by printing
/// `cargo:rustc-env=NATIVE_VERSIONING_VERSION=$value` to `stdout`.
pub fn write_versioned_header<I, H>(
    include_dir: I,
    header_filename: H,
    macro_name: &str,
) -> Result<PathBuf, Error>
    where I: AsRef<Path>, H: AsRef<Path>
{
    let include_dir = include_dir.as_ref();
    let versioned_h = include_dir.join(header_filename.as_ref());
    let version = version()?;

    fs::create_dir_all(include_dir)?;
    let mut file = File::create(&versioned_h)?;
    write!(file, "#define {}(sym) sym ## _{}\n", macro_name, version)?;
    println!("cargo:rustc-env={}={}", ENV_NAME, version);

    Ok(versioned_h)
}
