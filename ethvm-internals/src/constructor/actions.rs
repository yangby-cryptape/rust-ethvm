// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2;

use caches;
use definition;

pub struct Constructor {
    name: syn::Ident,
    actions: Vec<syn::Ident>,
    action_impls: Vec<proc_macro2::TokenStream>,
    unknown: syn::Block,
}

impl Constructor {
    pub fn new(action_group: definition::ActionGroup) -> Self {
        let definition::ActionGroup {
            name,
            actions,
            unknown,
        } = action_group;
        let action_unknown = syn::Ident::new("exec_unknown", proc_macro2::Span::call_site());
        let mut action_array: Vec<syn::Ident> = vec![action_unknown; 256];
        let mut action_impls: Vec<proc_macro2::TokenStream> = Vec::new();
        for action in actions.into_iter() {
            let definition::Action { mnemonic, block } = action;
            let mnemonic_string = &mnemonic.to_string();
            let mut value = 0;
            caches::OPCODE_TABLE.with(|f| {
                value = *(*f.borrow_mut())
                    .entry(mnemonic_string.clone())
                    .or_insert_with(|| {
                        panic!("the opcode `{}` has not been defined", mnemonic_string)
                    });
            });
            let action_name = format!("exec_{}", mnemonic_string.to_lowercase());
            let action_ident =
                syn::Ident::new(action_name.as_str(), proc_macro2::Span::call_site());
            action_array[value as usize] = action_ident.clone();
            let action_impl = quote!(#[inline] pub fn #action_ident () #block);
            action_impls.push(action_impl);
        }
        Constructor {
            name,
            actions: action_array,
            action_impls,
            unknown,
        }
    }

    fn output(&self) -> proc_macro2::TokenStream {
        let group_name = &self.name;
        let actions = &self.actions;
        let action_impls = &self.action_impls;
        let unknown = &self.unknown;
        let module_name = syn::Ident::new(
            self.name.to_string().to_lowercase().as_str(),
            proc_macro2::Span::call_site(),
        );
        let module_names = vec![&module_name; 256];
        quote!(
            pub const #group_name: [ActionFunc; 256] = [
                #(#module_names::#actions,)*
            ];
            mod #module_name {
                #(#action_impls)*
                #[inline]
                pub fn exec_unknown() #unknown
            }
        )
    }

    pub fn construct_all(&self) -> proc_macro2::TokenStream {
        self.output()
    }
}
