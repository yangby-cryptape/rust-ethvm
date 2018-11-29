// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2;
use syn;

#[derive(Clone)]
pub struct Instruction {
    pub value: syn::LitInt,
    pub mnemonic: syn::Ident,
    pub immediate_vec: Vec<syn::LitInt>,
}

impl syn::parse::Parse for Instruction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let value = content.parse()?;
        let _: Token![,] = content.parse()?;
        let mnemonic = content.parse()?;
        let immediate_vec = if content.parse::<Token![,]>().is_ok() {
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
                }).collect()
        } else {
            Vec::new()
        };
        Ok(Instruction {
            value,
            mnemonic,
            immediate_vec,
        })
    }
}

#[derive(Clone)]
pub struct InstructionSet {
    pub instructions: Vec<Instruction>,
}

impl syn::parse::Parse for InstructionSet {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let instructions: syn::punctuated::Punctuated<Instruction, Token![,]> =
            input.parse_terminated(syn::parse::Parse::parse)?;
        let instructions = instructions.into_iter().collect();
        Ok(InstructionSet { instructions })
    }
}

impl InstructionSet {
    pub fn for_each_construct<F, G>(&self, f: F, g: G) -> Vec<proc_macro2::TokenStream>
    where
        F: Fn(&syn::LitInt, &syn::Ident) -> proc_macro2::TokenStream,
        G: Fn(&syn::LitInt, &syn::Ident, &syn::LitInt) -> proc_macro2::TokenStream,
    {
        self.instructions
            .iter()
            .map(|inst| {
                let Instruction {
                    ref value,
                    ref mnemonic,
                    ref immediate_vec,
                } = inst;
                if immediate_vec.is_empty() {
                    f(value, mnemonic)
                } else {
                    // now, the size of immediate values only can be 0 or 1
                    let iv1_size = immediate_vec.first().unwrap();
                    g(value, mnemonic, iv1_size)
                }
            }).collect()
    }
}
