extern crate tempdir;
extern crate finally;

use std::path::{Path, PathBuf};
use std::env::args_os;
use std::process::Command;
use std::fs::{read_dir, remove_dir, rename};

use tempdir::TempDir;

mod unpack_format;
use unpack_format::UnpackFormat;

fn unpack(formats: &[UnpackFormat], path: &Path) {
    let file_name = match path.file_name() {
        Some(file_name) => file_name,
        None => panic!("no file component")
    };

    let format = UnpackFormat::find(formats, file_name).unwrap();

    let dir = match TempDir::new_in(".", ".unpack") {
        Ok(dir) => dir,
        Err(e) => panic!(e)
    };

    let mut adjusted_path = PathBuf::new();
    adjusted_path.push("..");
    adjusted_path.push(path);
    let mut cmd = Command::new(&format.invocation[0]);
    cmd.args(&format.invocation[1..]);
    cmd.arg(&adjusted_path);
    cmd.current_dir(dir.path());
    cmd.stdout(std::process::Stdio::null());
    let status = cmd.spawn().unwrap().wait().unwrap();
    if !status.success() {
        panic!("child exited with status {}", status);
    }
    let mut single_entry = None;
    let mut iter = read_dir(dir.path()).unwrap();

    if let Some(entry) = iter.next() {
        if iter.next().is_none() {
            single_entry = Some(entry.unwrap().path());
        }
    }

    if let Some(entry) = single_entry {
        let new_name = entry.components().next_back().unwrap().as_os_str();
        rename(&entry, new_name).unwrap();
        remove_dir(dir.path()).unwrap();
        println!("unpacked into \"{}\"", new_name.to_string_lossy());
    } else {
        let name_str = file_name.to_str().unwrap();
        let extension_offset = name_str.len() - format.extension.len();
        let dest = &name_str[ .. extension_offset];
        rename(dir.path(), dest).unwrap();
        println!("unpacked into \"{}\"", dest);
    }
    dir.into_path();
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let formats = [
        UnpackFormat {
            extension: ".zip".into(),
            invocation: vec!["unzip".into()]
        },
        UnpackFormat {
            extension: ".tar.gz".into(),
            invocation: vec!["tar".into(), "xfz".into()]
        },
        UnpackFormat {
            extension: ".tar".into(),
            invocation: vec!["tar".into(), "xf".into()]
        },
    ];

    for arg in args_os().skip(1) {
        unpack(&formats, arg.as_ref());
    }
}
