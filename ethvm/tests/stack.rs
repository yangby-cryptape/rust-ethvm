// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate ethvm;

use ethvm::stack::Stack;

#[test]
fn size() {
    let mut stack = Stack::default();
    {
        let data = [0u8; 33];
        let _ = stack.push(&data[..]);
        assert_eq!(stack.size(), 0);
    }
    let data = [0u8; 32];
    {
        assert_eq!(stack.size(), 0);
        for i in 1..=1024 {
            let _ = stack.push(&data[..]);
            assert_eq!(stack.size(), i);
        }
        let _ = stack.push(&data[..]);
        assert_eq!(stack.size(), 1024);
        for i in (0..=1023).rev() {
            let _ = stack.pop();
            assert_eq!(stack.size(), i);
        }
        let _ = stack.pop();
        assert_eq!(stack.size(), 0);
    }
    {
        let mut expected = 0;
        let _ = stack.push(&data[..]);
        expected += 1;
        assert_eq!(stack.size(), expected);
        let _ = stack.dup(0);
        assert_eq!(stack.size(), expected);
        for i in 1..=16 {
            let _ = stack.dup(i);
            expected += 1;
            assert_eq!(stack.size(), expected);
        }
        let _ = stack.dup(17);
        assert_eq!(stack.size(), expected);
    }
    {
        let expected = stack.size();
        for i in 1..=16 {
            let _ = stack.swap(i);
            assert_eq!(stack.size(), expected);
        }
    }
}

#[test]
fn push_limit() {
    let mut stack = Stack::default();
    let data = [0u8; 33];
    assert!(stack.push(&data[..]).is_err());
    let data = [0u8; 32];
    for _ in 0..1024 {
        assert!(stack.push(&data[..]).is_ok());
    }
    assert!(stack.push(&data[..]).is_err());
}

#[test]
fn push() {
    let mut stack = Stack::default();
    for i in 1u8..=16 {
        let data = [(16 - i) * 0x11; 32];
        let size = stack.size();
        let _ = stack.push(&data[..]);
        assert_eq!(stack.size(), size + 1);
        assert_eq!(stack.peek().unwrap(), &data[..]);
    }
}

#[test]
fn pop_limit() {
    let mut stack = Stack::default();
    let data = [0u8; 32];
    for _ in 0..1024 {
        let _ = stack.push(&data[..]);
    }
    for _ in 0..1024 {
        assert!(stack.pop().is_ok());
    }
    assert!(stack.pop().is_err());
}

#[test]
fn pop() {
    let mut stack = Stack::default();
    for i in 1u8..=16 {
        let data = [(16 - i) * 0x11; 32];
        let _ = stack.push(&data[..]);
    }
    for i in 1u8..=16 {
        let data = [(i - 1) * 0x11; 32];
        let size = stack.size();
        let popped = stack.pop().unwrap();
        assert_eq!(stack.size(), size - 1);
        assert_eq!(popped, data);
    }
}

#[test]
fn back() {
    let mut stack = Stack::default();
    assert!(stack.back(0).is_err());
    for i in 1u8..=16 {
        let data = [(16 - i) * 0x11; 32];
        let _ = stack.push(&data[..]);
        for j in 0..(i - 1) {
            assert!(stack.back(j as usize).is_ok());
        }
        assert!(stack.back(i as usize).is_err());
    }
    for i in 0u8..=15 {
        let data = [i * 0x11; 32];
        assert_eq!(stack.back(i as usize).unwrap(), &data[..]);
    }
    assert!(stack.back(16).is_err());
}

#[test]
fn peek() {
    let mut stack = Stack::default();
    assert!(stack.peek().is_err());
    for i in 1u8..=16 {
        let data = [(16 - i) * 0x11; 32];
        let _ = stack.push(&data[..]);
        assert_eq!(stack.peek().unwrap(), &data[..]);
    }
    for i in 1u8..=16 {
        let data = [(i - 1) * 0x11; 32];
        assert_eq!(stack.peek().unwrap(), &data[..]);
        let _ = stack.pop().unwrap();
    }
}

#[test]
fn dup_limit() {
    let mut stack = Stack::default();
    assert!(stack.dup(1).is_err());
    let data = [0u8; 32];
    let _ = stack.push(&data[..]);
    for i in 1..=16 {
        assert!(stack.dup(0).is_err());
        for j in i + 1..=17 {
            assert!(stack.dup(j).is_err());
        }
        assert!(stack.dup(i).is_ok());
    }
}

#[test]
fn dup() {
    let mut stack = Stack::default();
    for i in 1u8..=16 {
        let data = [(16 - i) * 0x11; 32];
        let _ = stack.push(&data[..]);
    }
    for i in 1u8..=16 {
        let size = stack.size();
        let _ = stack.dup(i as usize);
        assert_eq!(stack.size(), size + 1);
        let data = [(i - 1) * 0x11; 32];
        let popped = stack.pop().unwrap();
        assert_eq!(popped, data);
    }
}

#[test]
fn swap_limit() {
    let mut stack = Stack::default();
    let data = [0u8; 32];
    for i in 0u8..=17 {
        assert!(stack.swap(i as usize).is_err());
    }
    let _ = stack.push(&data[..]);
    for i in 0u8..=17 {
        assert!(stack.swap(i as usize).is_err());
    }
    for i in 1..=16 {
        let _ = stack.push(&data[..]);
        assert!(stack.swap(0).is_err());
        for j in 1..=i {
            assert!(stack.swap(j).is_ok());
        }
        for j in i + 1..=17 {
            assert!(stack.swap(j).is_err());
        }
    }
}

#[test]
fn swap() {
    let mut stack = Stack::default();
    let data = [0u8; 32];
    let _ = stack.push(&data[..]);
    for i in 1u8..=16 {
        let data = [(16 - i) * 0x11; 32];
        let _ = stack.push(&data[..]);
    }
    let size = stack.size();
    for i in 1u8..=15 {
        let data = [i * 0x11; 32];
        let _ = stack.swap(i as usize);
        assert_eq!(stack.size(), size);
        assert_eq!(stack.peek().unwrap(), &data[..]);
    }
    let _ = stack.swap(16);
    assert_eq!(stack.size(), size);
    assert_eq!(stack.peek().unwrap(), &data[..]);
}
