// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Provide some proc-macros.

#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

mod constructor;
mod definition;

/// Provide a proc-macro to create [`OpCode`] and [`OpCodes`].
///
/// [`OpCode`]: ../ethvm/enum.OpCode.html
/// [`OpCodes`]: ../ethvm/struct.OpCodes.html
#[proc_macro]
pub fn instruction_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inputs = parse_macro_input!(input as definition::InstructionSet);
    let expanded = {
        let constructor = constructor::Constructor::new(inputs.clone());
        constructor.construct_all()
    };
    expanded.into()
}
