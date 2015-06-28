extern crate tempdir;

#[macro_use]
extern crate error_type;

use std::path::{Path, PathBuf};
use std::env::args_os;
use std::process::Command;
use std::fs::{read_dir, remove_dir, rename};

use tempdir::TempDir;

mod unpack_format;
use unpack_format::UnpackFormat;
mod config;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BadPath {
    path: PathBuf,
}

impl BadPath {
    fn new<P: Into<PathBuf>>(p: P) -> BadPath {
        BadPath { path: p.into() }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BadFormat {
    path: PathBuf,
}

impl BadFormat {
    fn new<P: Into<PathBuf>>(p: P) -> BadFormat {
        BadFormat { path: p.into() }
    }
}

error_type! {
    #[derive(Debug)]
    pub enum UnpackError {
        Io(std::io::Error) {
            cause;
        },
        Arg(BadPath) {
            desc (_e) "bad path";
            disp (e, fmt)
                write!(fmt, "bad path: {}", e.path.to_string_lossy());
        },
        Format(BadFormat) {
            desc (_e) "bad format";
            disp (e, fmt)
                write!(fmt, "bad format: {}", e.path.to_string_lossy());
        },
        Child(std::process::ExitStatus) {
            desc (_e) "child exited unsuccessfully";
        }
    }
}

fn unpack(formats: &[UnpackFormat], path: &Path) -> Result<(), UnpackError> {
    let file_name = match path.file_name() {
        Some(file_name) => file_name,
        None => return Err(From::from(BadPath::new(path)))
    };

    let format = try!(UnpackFormat::find(formats, file_name).ok_or_else(
        || <UnpackError as From<_>>::from(BadFormat::new(path))));

    let dir = try!(TempDir::new_in(".", ".unpack"));

    let mut adjusted_path = PathBuf::new();
    adjusted_path.push("..");
    adjusted_path.push(path);
    let mut cmd = Command::new(&format.invocation[0]);
    cmd.args(&format.invocation[1..]);
    cmd.arg(&adjusted_path);
    cmd.current_dir(dir.path());
    cmd.stdout(std::process::Stdio::null());
    let status = try!(try!(cmd.spawn()).wait());
    if !status.success() {
        return Err(From::from(status));
    }
    let mut single_entry = None;
    let mut iter = try!(read_dir(dir.path()));

    if let Some(entry) = iter.next() {
        if iter.next().is_none() {
            single_entry = Some(try!(entry).path());
        }
    }

    if let Some(entry) = single_entry {
        let new_name = entry.components().next_back().unwrap().as_os_str();
        try!(rename(&entry, new_name));
        try!(remove_dir(dir.path()));
        println!("unpacked into \"{}\"", new_name.to_string_lossy());
    } else {
        let name_str = file_name.to_str().unwrap();
        let extension_offset = name_str.len() - format.extension.len();
        let dest = &name_str[ .. extension_offset];
        try!(rename(dir.path(), dest));
        println!("unpacked into \"{}\"", dest);
    }
    dir.into_path();

    Ok(())
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let formats = match config::load() {
        Ok(formats) => formats,
        Err(e) => panic!("error loading config: {:?}", e)
    };

    for arg in args_os().skip(1) {
        unpack(&formats, arg.as_ref()).unwrap();
    }
}
