// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syn;

#[derive(Clone)]
pub struct Action {
    pub mnemonic: syn::Ident,
    pub block: syn::Block,
}

impl syn::parse::Parse for Action {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![|] = input.parse()?;
        let mnemonic = input.parse()?;
        let _: Token![|] = input.parse()?;
        let block = input.parse()?;
        Ok(Action { mnemonic, block })
    }
}

#[derive(Clone)]
pub struct ActionGroup {
    pub name: syn::Ident,
    pub actions: Vec<Action>,
    pub unknown: syn::Block,
}

impl syn::parse::Parse for ActionGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let _ = bracketed!(content in input);
        let actions = {
            let actions: syn::punctuated::Punctuated<Action, Token![,]> =
                content.parse_terminated(syn::parse::Parse::parse)?;
            actions.into_iter().collect()
        };
        let _: Token![,] = input.parse()?;
        let unknown = input.parse()?;
        Ok(ActionGroup {
            name,
            actions,
            unknown,
        })
    }
}
