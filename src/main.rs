extern crate tempdir;

#[macro_use]
extern crate error_type;

use std::path::{Path, PathBuf};
use std::env::args_os;
use std::process;
use std::fs::{read_dir, remove_dir, rename};
use std::io::{Write, Cursor};
use std::ffi::{OsStr, OsString};

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MissingOperand;

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
        },
        Config(config::ConfigLoadError) {
            desc (_e) "couldn't load configuration";
            disp (e, fmt)
                write!(fmt, "couldn't load configuration: {}", e);
        },
        MissingOperand(MissingOperand) {
            desc (_e) "missing operand";
            disp (_e, fmt)
                write!(fmt, "missing operand");
        },
    }
}

fn unpack(formats: &[UnpackFormat], path: &Path) -> Result<(), UnpackError> {
    let file_name = match path.file_name() {
        Some(file_name) => file_name,
        None => return Err(From::from(BadPath::new(path)))
    };

    let format = try!(UnpackFormat::find(formats, file_name).ok_or_else(
        || UnpackError::from(BadFormat::new(path))));

    let dir = try!(TempDir::new_in(".", ".unpack"));

    let mut adjusted_path = PathBuf::new();
    adjusted_path.push("..");
    adjusted_path.push(path);

    let mut cmd = process::Command::new(&format.invocation[0]);
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

    fn find_suitable_name<'a>(base: &'a OsStr, os_string: &'a mut OsString)
    -> &'a OsStr {

        fn exists(s: &OsStr) -> bool {
            Path::symlink_metadata(s.as_ref()).is_ok()
        }

        if exists(base) {
            let mut buf = [0; 24];
            let mut buf = Cursor::new(&mut buf[..]);
            for counter in 2u32 .. {
                os_string.clear();
                os_string.push(base);

                buf.set_position(0);
                write!(&mut buf, "_{}", counter).unwrap();
                let suffix = &buf.get_ref()[0..(buf.position() as usize)];
                os_string.push(std::str::from_utf8(suffix).unwrap());

                if !exists(os_string.as_ref()) {
                    return &*os_string;
                }
            }

            panic!("couldn't find suitable output name as all names formed \
                    from the basename and a numeric suffix are already taken");
        } else {
            base
        }
    }

    let mut buf = OsString::new();
    if let Some(entry) = single_entry {
        let base_name = entry.components().next_back().unwrap().as_os_str();
        let new_name = find_suitable_name(base_name, &mut buf);
        try!(rename(&entry, new_name));
        try!(remove_dir(dir.path()));
        println!("unpacked into \"{}\"", new_name.to_string_lossy());
    } else {
        let name_str = file_name.to_str().unwrap();
        let extension_offset = name_str.len() - format.extension.len();
        let base_name = &name_str[ .. extension_offset];
        let new_name = find_suitable_name(base_name.as_ref(), &mut buf);
        try!(rename(dir.path(), new_name));
        println!("unpacked into \"{}\"", new_name.to_string_lossy());
    }
    dir.into_path();

    Ok(())
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    if let Err(e) = go() {
        let _ = write!(std::io::stderr(), "unpackrs: {}\n", e);
        process::exit(-1);
    }

    fn go() -> Result<(), UnpackError> {
        let formats = try!(config::load());

        let args = args_os().skip(1);

        if args.len() == 0 {
            return Err(From::from(MissingOperand));
        }

        for arg in args {
            try!(unpack(&formats, arg.as_ref()));
        }
        Ok(())
    }
}
