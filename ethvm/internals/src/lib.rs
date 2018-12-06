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

mod caches;

mod definition;

mod constructor;

/// Provide a proc-macro to create [`OpCode`] and [`OpCodeStmt`].
///
/// [`OpCode`]: ../ethvm/enum.OpCode.html
/// [`OpCodeStmt`]: ../ethvm/struct.OpCodeStmt.html
///
/// # Usage
///
/// ```ignore
/// define_opcodes!(
///     [
///         (0x00, STOP, [], 0, 0),
///         (0x01, ADD, [], 2, 1),
///         (0x02, MUL, [], 2, 1),
///         (0x03, SUB, [], 2, 1),
///         ... ...
///         (0x60, PUSH1, [1], 0, 1),
///         (0x61, PUSH2, [2], 0, 1),
///         (0x62, PUSH3, [3], 0, 1),
///         ... ...
///     ]
/// );
/// ```
///
/// The input for this macro is a list.
///
/// Each element in the list is a tuple:
/// - The 1st element in the tuple is the value of the opcode.
/// - The 2nd element is the mnemonic.
/// - The 3rd element is an array of immediate values's sizes.
/// - The 4th element is the size of the items removed from stack.
/// - The 5th element is the size of the additional items placed on the stack.
#[proc_macro]
pub fn define_opcodes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inputs = parse_macro_input!(input as definition::OpCodeSet);
    let expanded = {
        let constructor = constructor::OpCodesConstructor::new(inputs);
        constructor.construct_all()
    };
    expanded.into()
}

/// Provide a proc-macro to create a group of actions how to execute instructions.
///
/// # Usage
///
/// ```ignore
/// create_action_groups!(
///     GROUP_NAME,
///     [
///         |STOP| {
///             ...
///         },
///         |ADD| {
///             ...
///         },
///         ... ...
///     ],
///     {
///         ...
///     }
/// );
/// ```
///
/// The input for this macro is an ident, a list and a block.
///
/// The ident is the name of this action group.
///
/// Each element in the list is a closure expression.
/// But there is only one ident between `|`s, and it is the [`OpCode`].
///
/// The last block is the action for an unknown [`OpCode`].
///
/// [`OpCode`]: ../ethvm/enum.OpCode.html
#[proc_macro]
pub fn create_action_groups(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inputs = parse_macro_input!(input as definition::ActionGroup);
    let expanded = {
        let constructor = constructor::ActionsConstructor::new(inputs);
        constructor.construct_all()
    };
    expanded.into()
}
