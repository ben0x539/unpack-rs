use std;
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;

use unpack_format::UnpackFormat;

error_type! {
    #[derive(Debug)]
    pub enum ConfigLoadError {
        Io(std::io::Error) {
            cause;
        },
        ParseError(&'static str) {
            desc (e) e;
        }
    }
}

pub type MyResult<T> = Result<T, ConfigLoadError>;

const CONFIG_FILE_NAME: &'static str = "unpack-rs.conf";

pub fn load() -> MyResult<Vec<UnpackFormat>> {
    let config = try!(load_from_file()).unwrap_or_else(|| {
        vec![
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
        ]
    });

    Ok(config)
}

fn load_from_file() -> MyResult<Option<Vec<UnpackFormat>>> {
    let path = match get_path() {
        Some(path) => path,
        None => return Ok(None)
    };
    let mut file = match File::open(path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        r => try!(r)
    };
    let mut s = String::new();
    try!(file.read_to_string(&mut s));

    let cfg = try!(parse_config(&*s));

    Ok(Some(cfg))
}

fn get_path() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME").map(|config| {
        let mut p = PathBuf::new();
        p.push(config);
        p
    }).or_else(|| {
        env::var_os("HOME").map(|home| {
            let mut p = PathBuf::new();
            p.push(home);
            p.push(".config");
            p
        })
    }).map(|mut p| {
        p.push(CONFIG_FILE_NAME);
        p
    })
}

fn parse_config(_s: &str) -> MyResult<Vec<UnpackFormat>> {
    Ok(vec![])
}
