//
// Copyright 2019 Hans W. Uhlig. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use super::{consts, TelnetError};
use bytes::{Buf, BufMut};

///
/// `NAOHTS` contains Negotiation data about Output Horizontal Tabstops.
/// [RFC653](http://www.iana.org/go/rfc653)
/// TODO: Implement This
///
#[derive(Clone, Debug, PartialEq)]
pub struct NAOHTS(Vec<u8>);

impl NAOHTS {
    pub fn new(data: Vec<u8>) -> NAOHTS {
        NAOHTS(data)
    }
    /// Get Encoded Length of `NAOHTS`
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    /// Encode `NAOHTS` to `BufMut`
    pub(crate) fn encode<T: BufMut>(&self, dst: &mut T) -> Result<(), TelnetError> {
        dst.put_u8(consts::option::NAOHTS);
        dst.put(self.0.as_slice());
        Ok(())
    }
    /// Decode `NAOCRD` from `Buf`
    pub fn decode<T: Buf>(src: &mut T) -> Result<Self, TelnetError> {
        Ok(NAOHTS::default())
    }
}

impl Default for NAOHTS {
    fn default() -> NAOHTS {
        NAOHTS::new(Vec::new())
    }
}