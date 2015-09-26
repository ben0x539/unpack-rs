use std;
use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::ErrorKind;
use std::io::BufReader;
use std::io::BufRead;
use std::iter::FromIterator;

use unpack_format::UnpackFormat;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum BadLine {
    NoColon,
    EmptyExtension,
    EmptyInvocation
}

impl BadLine {
    fn desc(&self) -> &'static str {
        match *self {
            BadLine::NoColon =>
                "bad line in config file: \
                 missing colon separating extension \
                 from unpack invocation",
            BadLine::EmptyExtension =>
                "bad line in config file: \
                 no extension given",
            BadLine::EmptyInvocation =>
                "bad line in config file: \
                 no invocation given after extension"
        }
    }
}

error_type! {
    #[derive(Debug)]
    pub enum ConfigLoadError {
        Io(std::io::Error) {
            cause;
        },
        ParseError((String, BadLine)) {
            desc (e) e.1.desc();
            disp (e, fmt)
                write!(fmt, "{}\nline was: {}", e.1.desc(), e.0);
        }
    }
}

pub type MyResult<T> = Result<T, ConfigLoadError>;

const CONFIG_FILE_PATH: &'static str = "unpack-rs/formats";

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
    let mut p = if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        config.into()
    } else if let Some(home) = env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".config");
        p
    } else {
        return None;
    };

    p.push(CONFIG_FILE_PATH);
    Some(p)
}

fn parse_format_line(line: String) -> MyResult<UnpackFormat> {
    return parse(&*line).map_err(|e| (line, e).into());

    fn parse(line: &str) -> Result<UnpackFormat, BadLine>  {
        let p = try!(line.find(':').ok_or(BadLine::NoColon));
        if line[..p].trim() == "" {
            return Err(BadLine::EmptyExtension);
        }

        let q = p + 1 + try!(
            line[p + 1..].find(|c: char| !c.is_whitespace())
                .ok_or(BadLine::EmptyInvocation));

        let r = line[q..].find('#').map(|r| r + q).unwrap_or(line.len());
        if q == r { return Err(BadLine::EmptyInvocation); };

        let ext = line[0..p].trim();
        let rest = &line[q..r].trim();
        // todo: maybe take shell expansion semantics into account and support
        // non-final path parameter positions?
        let tokens = rest.split(char::is_whitespace);
        let invocation = tokens.filter(|&s| s != "").map(Into::into).collect();

        Ok(UnpackFormat { extension: ext.into(), invocation: invocation })
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
        assert!(parse_format_line("foo: #".into()).is_err());
        assert!(parse_format_line(": bar".into()).is_err());
        let sample = 
            UnpackFormat { extension: "x".into(), invocation: vec!["y".into(), "z".into()] };
        assert_eq!(parse_format_line("x: y z".into()).unwrap(),
            sample);
        assert_eq!(parse_format_line(" x  :    y   z # aaa ".into()).unwrap(),
            sample);
    }
}
