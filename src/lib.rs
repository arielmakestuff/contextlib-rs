// contextlib-rs/src/lib.rs
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
use std::ops::DerefMut;
use std::rc::Rc;

// Third-party imports

// Local imports


// ===========================================================================
// Modules
// ===========================================================================


pub mod error;
use self::error::{ContextError, ContextErrorType, GenericError};


// ===========================================================================
// Globals
// ===========================================================================


type ContextResult = Result<(), ContextError>;


// type WithResult<C> = Result<C, ContextError>;


// ===========================================================================
// Traits
// ===========================================================================


pub trait Context {
    fn enter(&mut self) -> ContextResult { Ok(()) }
    fn exit(&mut self, err: &ContextResult) -> bool;
}


pub trait IterContext: Context + Iterator<Item=ContextResult> {

    fn enter(&mut self) -> ContextResult {
        match self.next() {
            Some(result) => result,
            None => {
                let err = ContextError::new(ContextErrorType::IterEnterError,
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


pub fn with<C, B>(context: &mut C, block: B) -> ContextResult
    where C: Context,
          B: FnOnce(&mut C) -> ContextResult {

    context.enter()?;
    let result = block(context);
    match context.exit(&result) {
        true => Ok(()),
        false => result
    }
}



pub struct ExitStack {
    stack: Vec<Box<Rc<Context>>>,
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
        self.stack.push(Box::new(ref_ctx));
        Ok(())
    }
}


impl Context for ExitStack {
    fn exit(&mut self, err: &ContextResult) -> bool {
        let mut handled_error = false;
        for boxed_rc in self.stack.iter_mut().rev() {
            let mut ctxrc = boxed_rc.deref_mut();
            let mut ctx = Rc::get_mut(&mut ctxrc).unwrap();
            if let true = ctx.exit(err) {
                handled_error = true;
            }
        }
        handled_error
    }
}


// ===========================================================================
// Unit tests
// ===========================================================================


#[cfg(test)]
mod tests {

    // use super::{Context, ContextResult, with, ContextError, ContextErrorType};
    use super::*;
    use super::error::{ContextErrorType, GenericError};

    #[test]
    fn simple() {
        pub struct Temp {
            orig: u8,
            val: u8,
        }

        impl Temp {
            fn new() -> Self {
                Temp { orig: 0, val: 0 }
            }
        }

        impl Context for Temp {
            fn enter(&mut self) -> ContextResult {
                self.orig = self.val;
                self.val = 42;
                Ok(())
            }
            fn exit(&mut self, err: &ContextResult) -> bool {
                self.val = self.orig;
                match err { _ => false }
            }
        }

        let mut t = Temp::new();
        assert_eq!(t.val, 0);

        let result = with(&mut t, |t| {
            assert_eq!(t.val, 42);
            Err(ContextError::new(ContextErrorType::Other, "Y U get Err?!"))
        });
        match result {
            Ok(()) => assert_eq!(t.val, 0),
            Err(err) => {
                let errmsg = format!("Error should not have occurred: {}", err);
                panic!(errmsg);
            }
        }
    }


    // #[test]
    // fn changedir() {
    //     use std::io::{Error, ErrorKind};

    //     pub struct ChangeDirectory {
    //         orig: PathBuf,
    //         cur: PathBuf,
    //         error: ContextResult<Error>,
    //     }

    //     impl Temp {
    //         fn new() -> Self {
    //             Temp { orig: 0, val: 0, error: Ok(()) }
    //         }
    //     }

    //     impl Context<String> for Temp {
    //         fn enter(&mut self) {
    //             self.orig = self.val;
    //             self.val = 42;
    //         }
    //         fn exit(&mut self) -> bool {
    //             self.val = self.orig;
    //             false
    //         }
    //         fn seterror(&mut self, err: ContextResult<String>) {
    //             self.error = err;
    //         }
    //         fn error(&self) -> ContextResult<String> {
    //             match &self.error {
    //                 &Err(ref e) => Err(e.clone()),
    //                 _ => Ok(())
    //             }
    //         }
    //     }

    //     let t = Temp::new();
    //     assert_eq!(t.val, 0);

    //     let result = with(t, |ref mut t| {
    //         assert_eq!(t.val, 42);
    //         Ok(())
    //     });
    //     match result {
    //         Ok(context) => assert_eq!(context.val, 0),
    //         Err((_, err)) => {
    //             let errmsg = format!("Error should not have occurred: {}", err);
    //             panic!(errmsg);
    //         }
    //     }
    // }

}


// ===========================================================================
//
// ===========================================================================

