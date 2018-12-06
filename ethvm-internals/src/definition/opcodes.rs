// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2;
use syn;

use caches;

#[derive(Clone)]
pub struct OpCode {
    pub value: syn::LitInt,
    pub mnemonic: syn::Ident,
    pub immediate_vec: Vec<syn::LitInt>,
    // the items removed from stack
    pub delta: syn::LitInt,
    // the additional items placed on the stack
    pub alpha: syn::LitInt,
}

impl syn::parse::Parse for OpCode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let value = content.parse()?;
        let _: Token![,] = content.parse()?;
        let mnemonic = content.parse()?;
        let _: Token![,] = content.parse()?;
        let immediate_vec = {
            let content_immediate;
            let _ = bracketed!(content_immediate in content);
            let immediate_vec: syn::punctuated::Punctuated<syn::LitInt, Token![,]> =
                content_immediate.parse_terminated(syn::parse::Parse::parse)?;
            immediate_vec
                .into_iter()
                .map(|i| {
                    syn::LitInt::new(
                        i.value(),
                        syn::IntSuffix::None,
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect()
        };
        let _: Token![,] = content.parse()?;
        let delta = content.parse()?;
        let _: Token![,] = content.parse()?;
        let alpha = content.parse()?;
        Ok(OpCode {
            value,
            mnemonic,
            immediate_vec,
            delta,
            alpha,
        })
    }
}

impl OpCode {
    pub fn value(&self) -> u8 {
        let value = self.value.value();
        if value > 255 {
            panic!(
                "The value ({}) of the OpCode({}) should in [0, 255].",
                value, self.mnemonic,
            );
        }
        value as u8
    }
}

#[derive(Clone)]
pub struct OpCodeSet {
    pub opcodes: Vec<OpCode>,
}

impl syn::parse::Parse for OpCodeSet {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = bracketed!(content in input);
        let opcodes = {
            let opcodes: syn::punctuated::Punctuated<OpCode, Token![,]> =
                content.parse_terminated(syn::parse::Parse::parse)?;
            opcodes
                .into_iter()
                .map(|opcode| {
                    let value = opcode.value();
                    let mnemonic = &opcode.mnemonic.to_string();
                    caches::OPCODE_TABLE.with(|f| {
                        (*f.borrow_mut())
                            .entry(mnemonic.clone())
                            .and_modify(|_| {
                                panic!("the opcode `{}` has been defined twice", mnemonic)
                            })
                            .or_insert_with(|| value);
                    });
                    caches::OPCODE_VALUE_TABLE.with(|f| {
                        (*f.borrow_mut())
                            .entry(value)
                            .and_modify(|mnemonic_old| {
                                panic!(
                                    "the value `{:#04x}` has been used twice ({} and {})",
                                    value, mnemonic_old, mnemonic
                                )
                            })
                            .or_insert_with(|| mnemonic.clone());
                    });
                    opcode
                })
                .collect()
        };
        Ok(OpCodeSet { opcodes })
    }
}

impl OpCodeSet {
    pub fn for_each_construct<F, G>(&self, f: F, g: G) -> Vec<proc_macro2::TokenStream>
    where
        F: Fn(&syn::LitInt, &syn::Ident, &syn::LitInt, &syn::LitInt) -> proc_macro2::TokenStream,
        G: Fn(
            &syn::LitInt,
            &syn::Ident,
            &syn::LitInt,
            &syn::LitInt,
            &syn::LitInt,
        ) -> proc_macro2::TokenStream,
    {
        self.opcodes
            .iter()
            .map(|opcode| {
                let OpCode {
                    ref value,
                    ref mnemonic,
                    ref immediate_vec,
                    ref delta,
                    ref alpha,
                } = opcode;
                if immediate_vec.is_empty() {
                    f(value, mnemonic, delta, alpha)
                } else {
                    // now, the size of immediate values only can be 0 or 1
                    let iv1_size = immediate_vec.first().unwrap();
                    g(value, mnemonic, delta, alpha, iv1_size)
                }
            })
            .collect()
    }
}
