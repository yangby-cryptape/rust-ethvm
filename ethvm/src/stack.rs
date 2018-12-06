// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// EVM stack.
///
/// In EVM, the stack has a maximum size of 1024, and size of stack items is 256-bit.
#[derive(Clone)]
pub struct Stack {
    data: [StackItem; 1024],
    ptr: usize,
    zeros: StackItem,
}

/// EVM stack item.
type StackItem = [u8; 32];

/// EVM stack errors.
#[derive(Debug)]
pub enum StackError {
    Underflow,
    Overflow,
    Internal,
}

impl ::std::default::Default for Stack {
    #[inline]
    fn default() -> Self {
        let data = [[0; 32]; 1024];
        let ptr = 0;
        let zeros = [0; 32];
        Self { data, ptr, zeros }
    }
}

impl ::std::fmt::Debug for Stack {
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "####################      Stack      ####################"
        )?;
        if self.ptr == 0 {
            writeln!(
                f,
                "                     ---- empty ----                     "
            )?;
        } else {
            for (n, i) in (0..self.ptr).rev().enumerate() {
                write!(f, "{:#05x}+00:", n)?;
                for v in &(self.data[i])[0..16] {
                    write!(f, " {:02x}", v)?;
                }
                writeln!(f)?;
                write!(f, "{:#05x}+0f:", n)?;
                for v in &(self.data[i])[16..32] {
                    write!(f, " {:02x}", v)?;
                }
                writeln!(f)?;
            }
        }
        writeln!(
            f,
            "#########################################################"
        )
    }
}

impl ::std::fmt::Display for Stack {
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        writeln!(f, "Stack {{ size = {} }}", self.ptr)
    }
}

impl Stack {
    #[inline]
    pub fn size(&self) -> usize {
        self.ptr
    }

    #[inline]
    pub fn push(&mut self, input: &[u8]) -> Result<(), StackError> {
        let input_length = input.len();
        if input_length > 32 {
            Err(StackError::Internal)
        } else if self.ptr >= 1024 {
            Err(StackError::Overflow)
        } else {
            let data = &mut self.data[self.ptr];
            (&mut data[32 - input_length..32]).copy_from_slice(&input[..]);
            self.ptr += 1;
            Ok(())
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Result<[u8; 32], StackError> {
        if self.ptr == 0 {
            Err(StackError::Underflow)
        } else {
            self.ptr -= 1;
            let ret = self.data[self.ptr];
            let data = &mut self.data[self.ptr];
            (&mut data[..]).copy_from_slice(&self.zeros[..]);
            Ok(ret)
        }
    }

    #[inline]
    pub fn back(&self, n: usize) -> Result<&[u8], StackError> {
        if self.ptr < n + 1 {
            Err(StackError::Underflow)
        } else {
            let data = &self.data[self.ptr - n - 1];
            Ok(&data[..])
        }
    }

    #[inline]
    pub fn peek(&self) -> Result<&[u8], StackError> {
        self.back(0)
    }

    #[inline]
    pub fn dup(&mut self, n: usize) -> Result<(), StackError> {
        if 1 > n || n > 16 {
            Err(StackError::Internal)
        } else if self.ptr >= 1024 {
            Err(StackError::Overflow)
        } else if self.ptr < n {
            Err(StackError::Underflow)
        } else {
            let (left, right) = self.data.split_at_mut(self.ptr);
            let x = &mut right[0];
            let y = &left[self.ptr - n];
            (x[..]).copy_from_slice(&y[..]);
            self.ptr += 1;
            Ok(())
        }
    }

    #[inline]
    pub fn swap(&mut self, n: usize) -> Result<(), StackError> {
        if 1 > n || n > 16 {
            Err(StackError::Internal)
        } else if self.ptr < n + 1 {
            Err(StackError::Underflow)
        } else {
            let (left, right) = self.data.split_at_mut(self.ptr - 1);
            let x = &mut right[0];
            let y = &mut left[self.ptr - n - 1];
            (x[..]).swap_with_slice(y);
            Ok(())
        }
    }
}
