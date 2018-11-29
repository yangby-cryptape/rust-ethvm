// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use definition;
use proc_macro2;

use std::cell::Cell;
use std::iter::FromIterator;

pub struct Constructor {
    insts: definition::InstructionSet,
    output: Cell<Vec<proc_macro2::TokenStream>>,
    opcodes_impls: Cell<Vec<proc_macro2::TokenStream>>,
}

impl Constructor {
    pub fn new(insts: definition::InstructionSet) -> Self {
        let output = Cell::new(Vec::new());
        let opcodes_impls = Cell::new(Vec::new());
        Constructor {
            insts,
            output,
            opcodes_impls,
        }
    }

    fn append(&self, part: proc_macro2::TokenStream) {
        let mut ts_vec = self.output.take();
        ts_vec.push(part);
        self.output.set(ts_vec);
    }

    fn impl_opcodes(&self, part: proc_macro2::TokenStream) {
        let mut ts_vec = self.opcodes_impls.take();
        ts_vec.push(part);
        self.opcodes_impls.set(ts_vec);
    }

    fn output(&self) -> proc_macro2::TokenStream {
        let outputs = proc_macro2::TokenStream::from_iter(self.output.take());
        let opcodes_impls = proc_macro2::TokenStream::from_iter(self.opcodes_impls.take());
        quote!(
            #outputs
            impl OpCodes {
                #opcodes_impls
            }
        )
    }

    fn clear(&self) {
        let _ = self.output.take();
        let _ = self.opcodes_impls.take();
    }

    pub fn construct_all(&self) -> proc_macro2::TokenStream {
        self.clear();
        self.def_error();
        self.def_definition();
        self.defun_utils();
        self.impl_std_fmt_display();
        self.impl_std_convert_into_bytes();
        self.impl_std_str_fromstr();
        self.impl_opcodes_all();
        self.output()
    }

    fn def_error(&self) {
        let part = quote!(
            // All errors for instructions.
            mod error {
                #[derive(Debug, Clone, Copy)]
                pub enum FromSlice {
                    BadSizeSince(usize),
                    BadInstruction(usize, u8),
                }
                #[derive(Debug, Clone, Copy)]
                pub enum FromHex {
                    BadSize,
                    BadHexAt(usize),
                }
                #[derive(Debug, Clone, Copy)]
                pub enum FromHexStr {
                    BadSize,
                    BadHexAt(usize),
                    BadSlice(FromSlice),
                }
                #[derive(Debug, Clone)]
                pub enum FromStr {
                    BadHexSizeFor(usize),
                    BadHexFor(usize),
                    BadHexAt(usize, String, usize),
                    BadInstruction(usize, String),
                }
                impl ::std::convert::From<FromSlice> for FromHexStr {
                    #[inline]
                    fn from(err: FromSlice) -> Self {
                        FromHexStr::BadSlice(err)
                    }
                }
                impl ::std::convert::From<FromHex> for FromHexStr {
                    #[inline]
                    fn from(err: FromHex) -> Self {
                        match err {
                            FromHex::BadSize => FromHexStr::BadSize,
                            FromHex::BadHexAt(idx) => FromHexStr::BadHexAt(idx),
                        }
                    }
                }
            }
        );
        self.append(part);
    }

    fn def_definition(&self) {
        let core = &self.insts.for_each_construct(
            |_value, mnemonic| quote!(#mnemonic),
            |_value, mnemonic, iv1_size| quote!(#mnemonic([u8; #iv1_size])),
        );
        let part = quote!(
            /// Include an instruction and its immediate values if exist.
            ///
            /// More details can be found in the chapter Appendix H. Virtual Machine Specification
            /// of [Ethereum Yellow Paper].
            ///
            /// Defined by the proc-macro [`instruction_set`].
            ///
            /// [`instruction_set`]: ../ethvm_internals/fn.instruction_set.html
            /// [Ethereum Yellow Paper]: https://ethereum.github.io/yellowpaper/paper.pdf
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum OpCode {
                #(#core,)*
                BAD(u8),
            }

            /// A sequence of [`OpCode`].
            ///
            /// Defined by the proc-macro [`instruction_set`].
            ///
            /// [`OpCode`]: ./enum.OpCode.html
            /// [`instruction_set`]: ../ethvm_internals/fn.instruction_set.html
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct OpCodes (Vec<OpCode>);
        );
        self.append(part);
    }

    fn defun_utils(&self) {
        let part = quote!(#[inline]
        fn hexstr_to_bytes(s: &str) -> Result<Vec<u8>, self::error::FromHex> {
            let len = s.len();
            if len % 2 != 0 {
                return Err(self::error::FromHex::BadSize);
            }
            let mut ret = vec![0; len / 2];
            for (idx, chr) in s.bytes().enumerate() {
                let val = match chr {
                    b'a'...b'f' => chr - b'a' + 10,
                    b'A'...b'F' => chr - b'A' + 10,
                    b'0'...b'9' => chr - b'0',
                    _ => return Err(self::error::FromHex::BadHexAt(idx)),
                };
                if idx % 2 == 0 {
                    ret[idx / 2] |= val << 4;
                } else {
                    ret[idx / 2] |= val;
                }
            }
            Ok(ret)
        });
        self.append(part);
    }

    fn impl_std_fmt_display(&self) {
        let core = &self.insts.for_each_construct(
            |_value, mnemonic| {
                quote!(
                    OpCode::#mnemonic => {
                        write!(f, stringify!(#mnemonic))?;
                    }
                )
            },
            |_value, mnemonic, _iv1_size| {
                quote!(
                    OpCode::#mnemonic(ref iv) => {
                        write!(f, stringify!(#mnemonic))?;
                        write!(f, " 0x")?;
                        for i in &iv[..] {
                            write!(f, "{:02x}", i)?;
                        }
                    }
                )
            },
        );
        let part = quote!(
            impl ::std::fmt::Display for OpCode {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match *self {
                        #(#core)*
                        OpCode::BAD(v) => write!(f, "BAD {:#x}", v)?,
                    }
                    Ok(())
                }
            }
            impl ::std::fmt::Display for OpCodes {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    for oc in self.0.iter() {
                        match *oc {
                            #(#core)*
                            OpCode::BAD(v) => write!(f, "BAD {:#x}", v)?,
                        }
                        writeln!(f)?;
                    }
                    Ok(())
                }
            }
        );
        self.append(part);
    }

    fn impl_std_convert_into_bytes(&self) {
        let core = &self.insts.for_each_construct(
            |value, mnemonic| quote!(OpCode::#mnemonic => ret.push(#value),),
            |value, mnemonic, _iv1_size| {
                quote!(
                    OpCode::#mnemonic(iv) => {
                        ret.push(#value);
                        ret.extend_from_slice(&iv[..]);
                    },
                )
            },
        );
        let part = quote!(
            impl<'a> ::std::convert::From<&'a OpCodes> for Vec<u8> {
                #[inline]
                fn from(ocs: &OpCodes) -> Self {
                    let OpCodes (ref ocs) = ocs;
                    let mut ret = Vec::with_capacity(ocs.len()+32*16);
                    for oc in ocs.iter() {
                        match *oc {
                            #(#core)*
                            OpCode::BAD(v) => ret.push(v),
                        }
                    }
                    ret
                }
            }
        );
        self.append(part);
    }

    fn impl_std_str_fromstr(&self) {
        let core = &self.insts.for_each_construct(
            |_value, mnemonic| {
                quote!(
                    stringify!(#mnemonic) => {
                        idx += 1;
                        OpCode::#mnemonic
                    }
                )
            },
            |_value, mnemonic, iv1_size| {
                quote!(
                    stringify!(#mnemonic) => {
                        idx += 1;
                        let p = s[idx];
                        let len = p.len();
                        if 2 >= len || len > #iv1_size * 2 + 2 {
                            return Err(self::error::FromStr::BadHexSizeFor(idx));
                        }
                        if &p[0..2] != "0x" {
                            return Err(self::error::FromStr::BadHexFor(idx));
                        }
                        let mut iv = [0u8; #iv1_size];
                        let mut j = #iv1_size;
                        let mut high = false;
                        for (chr_idx, chr) in p[2..].bytes().enumerate().rev() {
                            let v = match chr {
                                b'a'...b'f' => chr - b'a' + 10,
                                b'A'...b'F' => chr - b'A' + 10,
                                b'0'...b'9' => chr - b'0',
                                _ => return Err(
                                    self::error::FromStr::BadHexAt(idx, p.to_owned(), chr_idx+2)
                                ),
                            };
                            if high {
                                iv[j] += v * 16;
                                high = false;
                            } else {
                                j -= 1;
                                iv[j] = v;
                                high = true;
                            }
                        }
                        idx += 1;
                        OpCode::#mnemonic(iv)
                        },
                )
            },
        );
        let part = quote!(
            impl ::std::str::FromStr for OpCodes {
                type Err = self::error::FromStr;
                #[inline]
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let s = s.split_whitespace()
                        .map(|x| {
                            if x == "KECCAK256" {
                                "SHA3"
                            }
                            else {
                                x
                            }
                        }).collect::<Vec<_>>();
                    let len = s.len();
                    let mut ret = Vec::with_capacity(len+32*16);
                    let mut idx = 0;
                    while idx < len {
                        let oc = match s[idx] {
                            #(#core)*
                            other => {
                                if other == "BAD" {
                                    idx += 1;
                                }
                                let bad = s[idx];
                                let len = bad.len();
                                if 2 >= len || len > 4 {
                                    return Err(
                                        self::error::FromStr::BadInstruction(idx, bad.to_owned())
                                    );
                                }
                                if &bad[0..2] != "0x" {
                                    return Err(
                                        self::error::FromStr::BadInstruction(idx, bad.to_owned())
                                    );
                                }
                                let t = bad.as_bytes();
                                let x = {
                                    let chr = t[2];
                                    match chr {
                                        b'a'...b'f' => chr - b'a' + 10,
                                        b'A'...b'F' => chr - b'A' + 10,
                                        b'0'...b'9' => chr - b'0',
                                        _ => return Err(
                                            self::error::FromStr::BadInstruction(idx, bad.to_owned())
                                        ),
                                    }
                                };
                                if len == 3 {
                                    idx += 1;
                                    OpCode::BAD(x)
                                } else {
                                    let y = {
                                        let chr = t[3];
                                        match chr {
                                            b'a'...b'f' => chr - b'a' + 10,
                                            b'A'...b'F' => chr - b'A' + 10,
                                            b'0'...b'9' => chr - b'0',
                                            _ => return Err(
                                                self::error::FromStr::BadInstruction(idx, bad.to_owned())
                                            ),
                                        }
                                    };
                                    idx += 1;
                                    OpCode::BAD(x*16+y)
                                }
                            },
                        };
                        ret.push(oc);
                    }
                    Ok(OpCodes(ret))
                }
            }
        );
        self.append(part);
    }

    fn impl_opcodes_all(&self) {
        let core = &self.insts.for_each_construct(
            |value, mnemonic| {
                quote!(
                    #value => {
                        idx += 1;
                        OpCode::#mnemonic
                    }
                )
            },
            |value, mnemonic, iv1_size| {
                quote!(
                    #value => {
                        idx += 1;
                        let mut iv = [0u8; #iv1_size];
                        let idx_new = idx + #iv1_size;
                        if idx_new > len {
                            (&mut iv[0..(len-idx)]).copy_from_slice(&slice[idx..]);
                        } else {
                            iv.copy_from_slice(&slice[idx..idx_new]);
                        }
                        idx = idx_new;
                        OpCode::#mnemonic(iv)
                    }
                )
            },
        );
        let part = quote!(
            /// Parse opcodes from str.
            #[inline]
            pub fn from_hex_str(string: &str) -> Result<Self, self::error::FromHexStr> {
                let slice = hexstr_to_bytes(string)?;
                let ocs = OpCodes::from_slice(&slice[..])?;
                Ok(ocs)
            }
            /// Parse opcodes from str, allow bad instructions.
            #[inline]
            pub fn from_hex_str_allow_bad(string: &str) -> Result<Self, self::error::FromHexStr> {
                let slice = hexstr_to_bytes(string)?;
                let ocs = OpCodes::from_slice_allow_bad(&slice[..])?;
                Ok(ocs)
            }
            /// Parse opcodes from slice.
            #[inline]
            pub fn from_slice(slice: &[u8]) -> Result<Self, self::error::FromSlice> {
                let len = slice.len();
                let mut ret = Vec::with_capacity(len);
                let mut idx = 0;
                while idx < len {
                    let oc = match slice[idx] {
                        #(#core)*
                        v => return Err(self::error::FromSlice::BadInstruction(idx, v)),
                    };
                    ret.push(oc);
                }
                Ok(OpCodes(ret))
            }
            /// Parse opcodes from slice, allow bad instructions.
            #[inline]
            pub fn from_slice_allow_bad(slice: &[u8]) -> Result<Self, self::error::FromSlice> {
                let len = slice.len();
                let mut ret = Vec::with_capacity(len);
                let mut idx = 0;
                while idx < len {
                    let oc = match slice[idx] {
                        #(#core)*
                        v => {
                            idx += 1;
                            OpCode::BAD(v)
                        }
                    };
                    ret.push(oc);
                }
                Ok(OpCodes(ret))
            }
        );
        self.impl_opcodes(part);
    }
}
