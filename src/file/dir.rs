use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

// ===========================================================================
// ** Directory **
// ===========================================================================

#[derive(Debug)]
pub struct Directory {
    pub path: String,
    pub entries: Vec<fs::DirEntry>,
}

impl Directory {
    // -----------------------------------------------------------------------
    // ** read **

    pub fn read(path: &str) -> Result<Self, io::Error> {
        // read the directory at 'path'.

        let entries = fs::read_dir(path)?;
        let mut dir_entries = Vec::new();

        // store all the valid DirEntrys.

        for entry in entries {
            if let Ok(entry) = entry {
                dir_entries.push(entry);
            }
        }

        // create the Directory struct.

        let directory = Directory {
            path: path.to_string(),
            entries: dir_entries,
        };

        Ok(directory)
    }

    // -----------------------------------------------------------------------
    // ** sanitize_path **

    pub fn sanitize_path(path: &str) -> String {
        let sanitized_path = if path.ends_with("/") || path.ends_with("\\") {
            format!("{}", path)
        } else {
            format!("{}/", path)
        };

        sanitized_path
    }

    // -----------------------------------------------------------------------
    // ** suffix **

    pub fn suffix(filename: &str) -> Option<&str> {
        let extension_opt = Path::new(filename).extension().and_then(OsStr::to_str);
        extension_opt
    }
}
