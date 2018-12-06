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
    opset: definition::OpCodeSet,
    output: Cell<Vec<proc_macro2::TokenStream>>,
    opcode_impls: Cell<Vec<proc_macro2::TokenStream>>,
    opstmt_impls: Cell<Vec<proc_macro2::TokenStream>>,
}

impl Constructor {
    pub fn new(opset: definition::OpCodeSet) -> Self {
        let output = Cell::new(Vec::new());
        let opcode_impls = Cell::new(Vec::new());
        let opstmt_impls = Cell::new(Vec::new());
        Constructor {
            opset,
            output,
            opcode_impls,
            opstmt_impls,
        }
    }

    fn append(&self, part: proc_macro2::TokenStream) {
        let mut ts_vec = self.output.take();
        ts_vec.push(part);
        self.output.set(ts_vec);
    }

    fn impl_opcode(&self, part: proc_macro2::TokenStream) {
        let mut ts_vec = self.opcode_impls.take();
        ts_vec.push(part);
        self.opcode_impls.set(ts_vec);
    }

    fn impl_opstmt(&self, part: proc_macro2::TokenStream) {
        let mut ts_vec = self.opstmt_impls.take();
        ts_vec.push(part);
        self.opstmt_impls.set(ts_vec);
    }

    fn output(&self) -> proc_macro2::TokenStream {
        let outputs = proc_macro2::TokenStream::from_iter(self.output.take());
        let opcode_impls = proc_macro2::TokenStream::from_iter(self.opcode_impls.take());
        let opstmt_impls = proc_macro2::TokenStream::from_iter(self.opstmt_impls.take());
        quote!(
            #outputs
            impl OpCode {
                #opcode_impls
            }
            impl OpCodeStmt {
                #opstmt_impls
            }
        )
    }

    fn clear(&self) {
        let _ = self.output.take();
        let _ = self.opcode_impls.take();
        let _ = self.opstmt_impls.take();
    }

    pub fn construct_all(&self) -> proc_macro2::TokenStream {
        self.clear();
        self.def_error();
        self.def_definition();
        self.defun_utils();
        self.impl_std_fmt_display();
        self.impl_std_convert_into_bytes();
        self.impl_std_str_fromstr();
        self.impl_std_iter();
        self.impl_opcode_const();
        self.impl_opstmt_convert();
        self.output()
    }

    fn def_error(&self) {
        let part = quote!(mod error {
            #[derive(Debug, Clone, Copy)]
            pub enum FromValueSlice {
                BadSizeSince(usize),
                UnknownValue(usize, u8),
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
                BadValueSlice(FromValueSlice),
            }
            #[derive(Debug, Clone)]
            pub enum FromStr {
                BadHexSizeFor(usize),
                BadHexFor(usize),
                BadHexAt(usize, String, usize),
                UnknownString(usize, String),
            }
            impl ::std::convert::From<FromValueSlice> for FromHexStr {
                #[inline]
                fn from(err: FromValueSlice) -> Self {
                    FromHexStr::BadValueSlice(err)
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
        });
        self.append(part);
    }

    fn def_definition(&self) {
        let core = &self.opset.for_each_construct(
            |_value, mnemonic, _delta, _alpha| quote!(#mnemonic),
            |_value, mnemonic, _delta, _alpha, iv1_size| quote!(#mnemonic([u8; #iv1_size])),
        );
        let part = quote!(
            /// Include an instruction and its immediate values if exist.
            ///
            /// More details can be found in the chapter Appendix H. Virtual Machine Specification
            /// of [Ethereum Yellow Paper].
            ///
            /// Defined by the proc-macro [`define_opcodes`].
            ///
            /// [`define_opcodes`]: ../ethvm_internals/fn.define_opcodes.html
            /// [Ethereum Yellow Paper]: https://ethereum.github.io/yellowpaper/paper.pdf
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum OpCode {
                #(#core,)*
                UNKNOWN(u8),
            }

            /// A sequence of [`OpCode`].
            ///
            /// Defined by the proc-macro [`define_opcodes`].
            ///
            /// [`OpCode`]: ./enum.OpCode.html
            /// [`define_opcodes`]: ../ethvm_internals/fn.define_opcodes.html
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct OpCodeStmt (Vec<OpCode>);
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
        let core = &self.opset.for_each_construct(
            |_value, mnemonic, _delta, _alpha| {
                quote!(
                    OpCode::#mnemonic => {
                        write!(f, stringify!(#mnemonic))?;
                    }
                )
            },
            |_value, mnemonic, _delta, _alpha, _iv1_size| {
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
                        OpCode::UNKNOWN(v) => write!(f, "UNKNOWN {:#x}", v)?,
                    }
                    Ok(())
                }
            }
            impl ::std::fmt::Display for OpCodeStmt {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    for opcode in self.0.iter() {
                        match *opcode {
                            #(#core)*
                            OpCode::UNKNOWN(v) => write!(f, "UNKNOWN {:#x}", v)?,
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
        let core = &self.opset.for_each_construct(
            |value, mnemonic, _delta, _alpha| quote!(OpCode::#mnemonic => ret.push(#value),),
            |value, mnemonic, _delta, _alpha, _iv1_size| {
                quote!(
                    OpCode::#mnemonic(iv) => {
                        ret.push(#value);
                        ret.extend_from_slice(&iv[..]);
                    },
                )
            },
        );
        let part = quote!(
            impl<'a> ::std::convert::From<&'a OpCodeStmt> for Vec<u8> {
                #[inline]
                fn from(opstmt: &OpCodeStmt) -> Self {
                    let OpCodeStmt (ref opcodes) = opstmt;
                    let mut ret = Vec::with_capacity(opcodes.len()+32*16);
                    for opcode in opcodes.iter() {
                        match *opcode {
                            #(#core)*
                            OpCode::UNKNOWN(v) => ret.push(v),
                        }
                    }
                    ret
                }
            }
        );
        self.append(part);
    }

    fn impl_std_str_fromstr(&self) {
        let core = &self.opset.for_each_construct(
            |_value, mnemonic, _delta, _alpha| {
                quote!(
                    stringify!(#mnemonic) => {
                        idx += 1;
                        OpCode::#mnemonic
                    }
                )
            },
            |_value, mnemonic, _delta, _alpha, iv1_size| {
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
            impl ::std::str::FromStr for OpCodeStmt {
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
                        let opcode = match s[idx] {
                            #(#core)*
                            other => {
                                if other == "UNKNOWN" {
                                    idx += 1;
                                }
                                let next = s[idx];
                                let len = next.len();
                                if 2 >= len || len > 4 {
                                    return Err(
                                        self::error::FromStr::UnknownString(idx, next.to_owned())
                                    );
                                }
                                if &next[0..2] != "0x" {
                                    return Err(
                                        self::error::FromStr::UnknownString(idx, next.to_owned())
                                    );
                                }
                                let t = next.as_bytes();
                                let x = {
                                    let chr = t[2];
                                    match chr {
                                        b'a'...b'f' => chr - b'a' + 10,
                                        b'A'...b'F' => chr - b'A' + 10,
                                        b'0'...b'9' => chr - b'0',
                                        _ => return Err(
                                            self::error::FromStr::UnknownString(idx, next.to_owned())
                                        ),
                                    }
                                };
                                if len == 3 {
                                    idx += 1;
                                    OpCode::UNKNOWN(x)
                                } else {
                                    let y = {
                                        let chr = t[3];
                                        match chr {
                                            b'a'...b'f' => chr - b'a' + 10,
                                            b'A'...b'F' => chr - b'A' + 10,
                                            b'0'...b'9' => chr - b'0',
                                            _ => return Err(
                                                self::error::FromStr::UnknownString(idx, next.to_owned())
                                            ),
                                        }
                                    };
                                    idx += 1;
                                    OpCode::UNKNOWN(x*16+y)
                                }
                            },
                        };
                        ret.push(opcode);
                    }
                    Ok(OpCodeStmt(ret))
                }
            }
        );
        self.append(part);
    }

    fn impl_std_iter(&self) {
        let part = quote!(
            impl ::std::iter::FromIterator<OpCode> for OpCodeStmt {
                #[inline]
                fn from_iter<I: IntoIterator<Item=OpCode>>(iter: I) -> Self {
                    let mut c = Vec::new();
                    for i in iter {
                        c.push(i);
                    }
                    OpCodeStmt(c)
                }
            }
            impl ::std::iter::IntoIterator for OpCodeStmt {
                type Item = OpCode;
                type IntoIter = ::std::vec::IntoIter<OpCode>;
                #[inline]
                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_iter()
                }
            }
        );
        self.append(part);
    }

    fn impl_opcode_const(&self) {
        let value = &self.opset.for_each_construct(
            |value, mnemonic, _delta, _alpha| quote!(OpCode::#mnemonic => #value),
            |value, mnemonic, _delta, _alpha, _iv1_size| quote!(OpCode::#mnemonic(..) => #value),
        );
        let delta = &self.opset.for_each_construct(
            |_value, mnemonic, delta, _alpha| quote!(OpCode::#mnemonic => #delta),
            |_value, mnemonic, delta, _alpha, _iv1_size| quote!(OpCode::#mnemonic(..) => #delta),
        );
        let alpha = &self.opset.for_each_construct(
            |_value, mnemonic, _delta, alpha| quote!(OpCode::#mnemonic => #alpha),
            |_value, mnemonic, _delta, alpha, _iv1_size| quote!(OpCode::#mnemonic(..) => #alpha),
        );
        let part = quote!(
            /// Get the value of an opcode.
            pub fn value(&self) -> u8 {
                match *self {
                    #(#value,)*
                    OpCode::UNKNOWN(val) => val,
                }
            }
            /// For each opcode, the items removed from stack.
            #[inline]
            pub fn stack_removed(&self) -> u8 {
                match *self {
                    #(#delta,)*
                    OpCode::UNKNOWN(_) => !0,
                }
            }
            /// For each opcode, the additional items placed on the stack.
            #[inline]
            pub fn stack_placed(&self) -> u8 {
                match *self {
                    #(#alpha,)*
                    OpCode::UNKNOWN(_) => !0,
                }
            }
        );
        self.impl_opcode(part);
    }

    fn impl_opstmt_convert(&self) {
        let core = &self.opset.for_each_construct(
            |value, mnemonic, _delta, _alpha| {
                quote!(
                    #value => {
                        idx += 1;
                        OpCode::#mnemonic
                    }
                )
            },
            |value, mnemonic, _delta, _alpha, iv1_size| {
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
            /// Parse `OpCodeStmt` from string.
            #[inline]
            pub fn from_hex_str(string: &str) -> Result<Self, self::error::FromHexStr> {
                let slice = hexstr_to_bytes(string)?;
                Ok(OpCodeStmt::from_value_slice(&slice[..])?)
            }
            /// Parse `OpCodeStmt` from string, allow unknown `OpCode`.
            #[inline]
            pub fn from_hex_str_allow_unknown(string: &str) -> Result<Self, self::error::FromHexStr> {
                let slice = hexstr_to_bytes(string)?;
                Ok(OpCodeStmt::from_value_slice_allow_unknown(&slice[..])?)
            }
            /// Parse `OpCodeStmt` from an `OpCode` value slice.
            #[inline]
            pub fn from_value_slice(slice: &[u8]) -> Result<Self, self::error::FromValueSlice> {
                let len = slice.len();
                let mut ret = Vec::with_capacity(len);
                let mut idx = 0;
                while idx < len {
                    let opcode = match slice[idx] {
                        #(#core)*
                        v => return Err(self::error::FromValueSlice::UnknownValue(idx, v)),
                    };
                    ret.push(opcode);
                }
                Ok(OpCodeStmt(ret))
            }
            /// Parse `OpCodeStmt` from an `OpCode` value slice, allow unknown `OpCode`.
            #[inline]
            pub fn from_value_slice_allow_unknown(slice: &[u8]) -> Result<Self, self::error::FromValueSlice> {
                let len = slice.len();
                let mut ret = Vec::with_capacity(len);
                let mut idx = 0;
                while idx < len {
                    let opcode = match slice[idx] {
                        #(#core)*
                        v => {
                            idx += 1;
                            OpCode::UNKNOWN(v)
                        }
                    };
                    ret.push(opcode);
                }
                Ok(OpCodeStmt(ret))
            }
            /// Convert `OpCodeStmt` to a `OpCode` slice.
            #[inline]
            pub fn as_slice(&self) -> &[OpCode] {
                &self.0[..]
            }
        );
        self.impl_opstmt(part);
    }
}
