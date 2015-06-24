use std::ffi::{OsString, OsStr};

#[derive(Debug, PartialEq, Eq)]
pub struct UnpackFormat {
    pub extension: String,
    pub invocation: Vec<OsString>
}

impl UnpackFormat {
    pub fn handles(&self, file_name: &OsStr) -> bool {
        let name_str = match file_name.to_str() {
            Some(s) => s,
            None => return false
        };
        let ext_len = self.extension.len();

        ext_len < name_str.len()
            && name_str[name_str.len() - ext_len ..] == self.extension
    }

    pub fn find<'a>(formats: &'a [Self], file_name: &OsStr) -> Option<&'a Self> {
        // TODO: what about names matching multiple formats?
        // eg. should this fn ensure foo.tar.gz matches *.tar.gz and not *.gz?
        for format in formats {
            if format.handles(file_name) {
                return Some(format);
            }
        }

        None
    }
}

#[cfg(test)]
mod test_unpack_format {
    use super::UnpackFormat;

    fn some_formats() -> Vec<UnpackFormat> {
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
            UnpackFormat {
                extension: ".gz".into(),
                invocation: vec!["gzip".into(), "-d".into()]
            },
        ]
    }

    fn find_extension<'a>(formats: &'a [UnpackFormat], file_name: &str) -> Option<&'a str> {
        let opt_format = UnpackFormat::find(formats, file_name.as_ref());

        opt_format.map(|x| x.extension.as_ref())
    }

    #[test]
    fn test_handles() {
        let formats = some_formats();
        assert_eq!(find_extension(&formats, "foo.zip"), Some(".zip"));
        assert_eq!(find_extension(&formats, "foo.tar.gz"), Some(".tar.gz"));
        assert_eq!(find_extension(&formats, "foo.gz"), Some(".gz"));
        assert_eq!(find_extension(&formats, ".zip"), None);
        // assert_eq!(find_extension(&formats, ".tar.gz"), None); ugh it's gz
    }
}
