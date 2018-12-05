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
///
/// # Usage
///
/// ```ignore
/// instruction_set![
///     (0x00, STOP, [], 0, 0),
///     (0x01, ADD, [], 2, 1),
///     (0x02, MUL, [], 2, 1),
///     (0x03, SUB, [], 2, 1),
///     ... ...
///     (0x60, PUSH1, [1], 0, 1),
///     (0x61, PUSH2, [2], 0, 1),
///     (0x62, PUSH3, [3], 0, 1),
///     ... ...
/// ];
/// ```
///
/// The input for this macro is a list.
///
/// Each element in the list is a tuple:
/// - The 1st element in the tuple is the value of the instruction.
/// - The 2nd element is the mnemonic.
/// - The 3rd element is an array of immediate values's sizes.
/// - The 4th element is the size of the items removed from stack.
/// - The 5th element is the size of the additional items placed on the stack.
#[proc_macro]
pub fn instruction_set(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inputs = parse_macro_input!(input as definition::InstructionSet);
    let expanded = {
        let constructor = constructor::Constructor::new(inputs.clone());
        constructor.construct_all()
    };
    expanded.into()
}
