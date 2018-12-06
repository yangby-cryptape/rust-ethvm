// Copyright (C) 2018 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local!(
    pub static OPCODE_TABLE: RefCell<HashMap<String, u8>> = RefCell::new(HashMap::new())
);

thread_local!(
    pub static OPCODE_VALUE_TABLE: RefCell<HashMap<u8, String>> = RefCell::new(HashMap::new())
);
