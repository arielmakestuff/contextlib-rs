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
use std::io;
use std::ops::DerefMut;
use std::path;

// Third-party imports

// Local imports
use super::{Context, ContextResult};
use super::util;


// ===========================================================================
// DropContext
// ===========================================================================


pub struct DropContext {
    context: Box<Context>
}


impl DropContext {

    pub fn new<T>(o: T) -> Self
        where T: Context + 'static {

        Self { context: Box::new(o) }
    }

}


impl Context for DropContext {

    fn enter(&mut self) -> ContextResult {
        let mut context = self.context.deref_mut();
        context.enter()
    }

    fn exit(&mut self, err: &ContextResult) -> bool {
        let mut context = self.context.deref_mut();
        match err { _ => context.exit(err) }
    }

}


impl Drop for DropContext {

    fn drop(&mut self) {
        let mut context = self.context.deref_mut();
        let err = Ok(());
        context.exit(&err);
    }
}



// ===========================================================================
// SwitchDir
// ===========================================================================


pub struct SwitchDir {
    orig_impl: DropContext
}


impl SwitchDir {

    pub fn new(dir: path::PathBuf) -> io::Result<Self> {
        let switcher = util::SwitchDir::new(dir)?;
        let mut context = DropContext::new(switcher);

        match context.enter() {
            Err(e) => {
                let errmsg = format!("{}", e);
                let err = io::Error::new(io::ErrorKind::Other, errmsg);
                Err(err)
            }
            _ => Ok(Self { orig_impl: context })
        }
    }

}


impl Context for SwitchDir {

    fn enter(&mut self) -> ContextResult {
        self.orig_impl.enter()
    }

    fn exit(&mut self, err: &ContextResult) -> bool {
        self.orig_impl.exit(err)
    }

}


// ===========================================================================
// Tests
// ===========================================================================


#[cfg(test)]
mod tests {
    extern crate chrono;
    extern crate tempdir;

    // --------------------
    // Helpers
    // --------------------

    use self::chrono::prelude::*;
    use self::tempdir::TempDir;
    use std::env::current_dir;

    use super::*;

    fn mktempdir() -> TempDir {
        let dt = UTC::now();
        let suffix = dt.format("%Y%m%d%H%M%S%.9f");
        let name = format!("contextlib-rs_test_{}", suffix.to_string());
        let tmpdir = TempDir::new(&name).unwrap();
        tmpdir
    }

    // --------------------
    // Tests
    // --------------------
    #[test]
    fn droputil_switchdir_changes_dir() {
        // GIVEN
        // current directory and a target directory
        let startdir = current_dir().unwrap();
        let tmpdir = mktempdir();
        let newdir = path::PathBuf::from(tmpdir.path());

        assert!(startdir != newdir);

        {
            // WHEN
            // the new directory is set as a context and switched to
            let newdir = SwitchDir::new(newdir);
            match newdir {
                Ok(_) => assert!(true),
                _ => assert!(false)
            }

            // THEN
            // the current directory has changed to the target when under the
            // context
            let curdir = current_dir().unwrap();
            assert_eq!(curdir, tmpdir.path());
        }

        // AND THEN
        // the current directory is changed back to the original dir once
        // context ends
        let curdir = current_dir().unwrap();
        assert_eq!(curdir, startdir);
    }
}


// ===========================================================================
//
// ===========================================================================
