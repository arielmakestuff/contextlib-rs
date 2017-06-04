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
// use std::ops::DerefMut;

// Third-party imports

// Local imports


// ===========================================================================
// Modules
// ===========================================================================

pub mod error;
use self::error::{ContextError};

pub mod util;
pub mod droputil;


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


// ===========================================================================
// with
// ===========================================================================


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


// ===========================================================================
// Unit tests
// ===========================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        // GIVEN: A new context named Temp
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

        // WHEN:
        // Temp context is entered
        let mut t = Temp::new();
        assert_eq!(t.val, 0);

        let result = with(&mut t, |t| {
            assert_eq!(t.val, 42);
            Ok(())
        });

        // THEN: the context value is set to 42 and the context value is set
        // back to 0 when the context ends
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

