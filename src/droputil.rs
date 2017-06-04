// contextlib-rs/src/droputil.rs
// Copyright (C) 2017 authors and contributors (see AUTHORS file)
//
// This file is released under the MIT License.

// ===========================================================================
// Externs
// ===========================================================================


// ===========================================================================
// Imports
// ===========================================================================


// Stdlib imports
use std::env::{current_dir, set_current_dir};
use std::io;
use std::path;

// Third-party imports

// Local imports


// ===========================================================================
// SwitchDir
// ===========================================================================


pub struct SwitchDir {
    original_dir: path::PathBuf
}


impl SwitchDir {

    pub fn new(dir: path::PathBuf) -> io::Result<Self> {
        // Ensure dir is a directory
        if let false = dir.is_dir() {
            let errmsg = format!("Not a directory: {}", dir.display());
            return Err(io::Error::new(io::ErrorKind::NotFound, errmsg));
        }

        let original_dir = current_dir()?;

        // Don't do anything if the new directory is the same as the current
        // directory
        if original_dir == dir {
            return Ok(Self { original_dir: original_dir });
        }

        // Change directories
        set_current_dir(dir.as_path())?;

        Ok(Self { original_dir: original_dir })
    }
}


impl Drop for SwitchDir {

    // Change back to original directory on drop
    // If an error occurs, it will be silently ignored
    fn drop(&mut self) {
        match set_current_dir(self.original_dir.as_path()) {
            _ => ()
        }
    }
}


// ===========================================================================
//
// ===========================================================================
