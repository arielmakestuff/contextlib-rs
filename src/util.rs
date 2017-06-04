// contextlib-rs/src/util.rs
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
use std::mem;
use std::path;
use std::rc::Rc;

// Third-party imports

// Local imports
use super::{Context, ContextResult};
use super::error::{ContextError, ContextErrorType, GenericError};


// ===========================================================================
// ContextDrop
// ===========================================================================


pub struct ContextDrop {
    obj: Option<Box<Drop>>
}


impl ContextDrop {

    pub fn new<T>(o: T) -> Self
        where T: Drop + 'static {

        Self { obj: Some(Box::new(o)) }
    }
}


impl Context for ContextDrop {

    fn exit(&mut self, err: &ContextResult) -> bool {
        let mut newval = None;
        mem::swap(&mut self.obj, &mut newval);
        if let Some(b) = newval {
            let b = &*b;
            drop(b);
        }
        match err { _=> false }
    }

}


// ===========================================================================
// IterContext
// ===========================================================================


pub trait IterContext: Context + Iterator<Item=ContextResult> {

    fn enter(&mut self) -> ContextResult {
        match self.next() {
            Some(result) => result,
            None => {
                let err = ContextError::new(ContextErrorType::EnterError,
                                            "None returned on enter");
                Err(err)
            }
        }
    }

    fn exit(&mut self, err: &ContextResult) -> bool {
        match self.next() {
            None => match err { _ => false },
            Some(_) => panic!("Context Iterator returned more than 1 value")
        }
    }
}


// ===========================================================================
// ExitCallback
// ===========================================================================


pub struct ExitCallback {
    callback: Rc<Fn(&ContextResult) -> bool>
}


impl ExitCallback {
    pub fn new<F>(f: F) -> Self
        where F: (Fn(&ContextResult) -> bool) + 'static {

        Self { callback: Rc::new(f) }
    }
}


impl Context for ExitCallback {
    fn exit(&mut self, err: &ContextResult) -> bool {
        let cb = self.callback.clone();
        cb(err)
    }
}


// ===========================================================================
// ExitStack
// ===========================================================================


pub struct ExitStack {
    stack: Vec<Rc<Context>>,
}


impl ExitStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn enter_context(&mut self, c: Rc<Context>) -> ContextResult {
        let mut ref_ctx = c.clone();
        {
            let mut ctx = Rc::get_mut(&mut ref_ctx).unwrap();
            ctx.enter()?;
        }
        self.push(ref_ctx);
        Ok(())
    }

    pub fn push(&mut self, c: Rc<Context>) {
        self.stack.push(c);
    }

    pub fn remove(&mut self, c: Rc<Context>) {
        let mut index: Option<usize> = None;
        for (i, context) in self.stack.iter().enumerate() {
            if let true = Rc::ptr_eq(context, &c) {
                index = Some(i);
                break
            }
        }

        if let Some(i) = index {
            self.stack.remove(i);
        }
    }

    pub fn callback<F>(&mut self, f: F) -> Rc<Context>
        where F: (Fn(&ContextResult) -> bool) + 'static {

        let context = Rc::new(ExitCallback::new(f));
        self.push(context.clone());
        context
    }

    pub fn pop_all(&mut self) -> ExitStack {
        let mut newstack = ExitStack::new();
        mem::swap(&mut self.stack, &mut newstack.stack);
        newstack
    }

    fn rollback(&mut self, err: &ContextResult) -> bool {
        let mut handled_error = false;
        for mut rc in self.stack.iter_mut().rev() {
            let mut ctx = Rc::get_mut(&mut rc).unwrap();
            if let true = ctx.exit(err) {
                handled_error = true;
            }
        }
        handled_error
    }

    pub fn close(&mut self) {
        let err = Ok(());
        self.rollback(&err);
    }
}


impl Context for ExitStack {
    fn exit(&mut self, err: &ContextResult) -> bool {
        let ret = self.rollback(err);
        self.stack = Vec::new();
        ret
    }
}


// ===========================================================================
// SwitchDir
// ===========================================================================


pub struct SwitchDir {
    original_dir: path::PathBuf,
    new_dir: path::PathBuf
}


impl SwitchDir {

    pub fn new(dir: path::PathBuf) -> io::Result<Self> {
        // Ensure dir is a directory
        if let false = dir.is_dir() {
            let errmsg = format!("Not a directory: {}", dir.display());
            return Err(io::Error::new(io::ErrorKind::NotFound, errmsg));
        }

        let ret = SwitchDir {
            original_dir: current_dir()?,
            new_dir: dir
        };

        Ok(ret)
    }
}


impl Context for SwitchDir {

    fn enter(&mut self) -> ContextResult {
        // Save the current directory
        match current_dir() {
            Err(_) => {
                let errmsg = "Could not get current directory";
                let err = ContextError::new(ContextErrorType::EnterError,
                                            &errmsg);
                return Err(err);
            },
            Ok(p) => {
                self.original_dir = p;
            }
        }

        // Don't do anything if the new directory is the same as the current
        // directory
        if self.original_dir == self.new_dir {
            return Ok(());
        }

        // Change directories
        let result = set_current_dir(self.new_dir.as_path());
        match result {
            Err(_) => {
                let errmsg = format!("Could not set directory: {}",
                                      self.new_dir.to_string_lossy());
                let err = ContextError::new(ContextErrorType::EnterError,
                                            &errmsg);
                Err(err)
            },
            _ => Ok(()),
        }
    }

    fn exit(&mut self, err: &ContextResult) -> bool {
        // Change back to original directory
        let result = set_current_dir(self.original_dir.as_path());
        match result {
            Err(_) => {
                let errmsg = format!("Could not set directory: {}",
                                      self.original_dir.to_string_lossy());
                panic!(errmsg);
            },
            _ => match err { _ => false}
        }
    }
}


// ===========================================================================
// Tests
// ===========================================================================


#[cfg(test)]
mod tests {

    // --------------------
    // Helpers
    // --------------------
    extern crate chrono;
    extern crate tempdir;
    use self::chrono::prelude::*;
    use self::tempdir::TempDir;

    use super::*;
    use super::super::with;

    fn mktempdir() -> TempDir {
        //Generate unique temp name
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
    fn util_switchdir_changes_dir() {
        // GIVEN
        // current directory and a target directory
        let startdir = current_dir().unwrap();
        let tmpdir = mktempdir();
        let newdir = path::PathBuf::from(tmpdir.path());

        assert!(startdir != newdir);

        // WHEN
        // the new directory is set as a context
        let mut newdir = SwitchDir::new(newdir).unwrap();

        // THEN
        // the current directory has changed to the target when under the
        // context
        with(&mut newdir, |_| {
            let curdir = current_dir().unwrap();
            assert_eq!(curdir, tmpdir.path());
            Ok(())
        }).unwrap();

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
