use std;
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::ErrorKind;
use std::io::BufReader;
use std::io::BufRead;
use std::iter::FromIterator;

use unpack_format::UnpackFormat;

error_type! {
    #[derive(Debug)]
    pub enum ConfigLoadError {
        Io(std::io::Error) {
            cause;
        },
        ParseError(String) {
            desc (_e) "bad line in config file";
            disp (e, fmt)
                write!(fmt, "bad line in config file: {}", e);
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
    let file = match File::open(path) {
        Err(ref e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        r => try!(r)
    };

    let mut formats: Vec<_> = try!(MyResult::from_iter(
        BufReader::new(file)
            .lines()
            .map(|line|
                line
                    .map_err(|e| e.into())
                    .and_then(parse_format_line))
    ));

    formats.sort_by(|a, b| Ord::cmp(&b.extension.len(), &a.extension.len()));

    Ok(Some(formats))
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

fn parse_format_line(line: String) -> MyResult<UnpackFormat> {
    return match try_to_parse(&*line) {
        Some(format) => Ok(format),
        None => Err(line.into())
    };

    fn try_to_parse(s: &str) -> Option<UnpackFormat> {
        split_at_colon_spaces(s).map(|(extension, invocation)| {
            let invocation =
                invocation.split(space).filter(|&s| s != "").map(Into::into).collect();
            UnpackFormat { extension: extension.into(), invocation: invocation }
        })
    }

    fn space(c: char) -> bool { c == ' ' || c == '\t' }

    fn split_at_colon_spaces(s: &str) -> Option<(&str, &str)> {
        s.find(':').and_then(|p| {
            let rest = &s[p + 1..];
            rest.find(|c| !space(c)).map(|q| (&s[0..p], &rest[q..]))
        })
    }
}

#[cfg(test)]
mod test {
    use super::parse_format_line;
    use unpack_format::UnpackFormat;

    #[test]
    fn test_parse() {
        assert!(parse_format_line("foo".into()).is_err());
        assert!(parse_format_line("foo:".into()).is_err());
        assert!(parse_format_line("foo: ".into()).is_err());
        assert_eq!(parse_format_line("x: y z".into()).unwrap(),
            UnpackFormat { extension: "x".into(), invocation: vec!["y".into(), "z".into()] });
    }
}
