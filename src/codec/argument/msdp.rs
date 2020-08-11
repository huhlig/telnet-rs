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

mod encoder;
mod parser;

use super::consts;
use std::collections::HashMap;
use crate::codec::TelnetError;

pub use self::encoder::{encode_msdp, encoded_msdp_length};

/// `MudServerData` contains data about the Mud.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MudServerData(HashMap<MudServerDataVariable, MudServerDataValue>);

impl From<Vec<(MudServerDataVariable, MudServerDataValue)>> for MudServerData {
    fn from(table: Vec<(MudServerDataVariable, MudServerDataValue)>) -> MudServerData {
        let mut map = HashMap::default();
        table.into_iter().for_each(|(k, v)| { map.insert(k, v); });
        MudServerData(map)
    }
}

impl From<HashMap<MudServerDataVariable, MudServerDataValue>> for MudServerData {
    fn from(table: HashMap<MudServerDataVariable, MudServerDataValue>) -> MudServerData {
        MudServerData(table)
    }
}

impl Default for MudServerData {
    fn default() -> MudServerData {
        MudServerData(HashMap::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MudServerDataVariable {
    Bytes(Vec<u8>),
    String(String),
}

impl From<&[u8]> for MudServerDataVariable {
    fn from(var: &[u8]) -> MudServerDataVariable {
        if let Ok(str) = std::str::from_utf8(var) {
            MudServerDataVariable::String(str.to_string())
        } else {
            MudServerDataVariable::Bytes(var.to_owned())
        }
    }
}

///
/// `MudServerDataValue`
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MudServerDataValue {
    Bytes(Vec<u8>),
    String(String),
    Array(Vec<MudServerDataValue>),
    Table(HashMap<MudServerDataVariable, MudServerDataValue>),
}

impl From<&[u8]> for MudServerDataValue {
    fn from(val: &[u8]) -> MudServerDataValue {
        if let Ok(str) = std::str::from_utf8(val) {
            MudServerDataValue::String(str.to_string())
        } else {
            MudServerDataValue::Bytes(val.to_owned())
        }
    }
}

impl From<&str> for MudServerDataValue {
    fn from(value: &str) -> MudServerDataValue {
        MudServerDataValue::String(value.to_string())
    }
}

impl From<Vec<MudServerDataValue>> for MudServerDataValue {
    fn from(array: Vec<MudServerDataValue>) -> MudServerDataValue {
        MudServerDataValue::Array(array)
    }
}

impl From<Vec<(MudServerDataVariable, MudServerDataValue)>> for MudServerDataValue {
    fn from(table: Vec<(MudServerDataVariable, MudServerDataValue)>) -> MudServerDataValue {
        let mut map = HashMap::default();
        table.into_iter().for_each(|(k, v)| { map.insert(k, v); });
        MudServerDataValue::Table(map)
    }
}

impl From<HashMap<MudServerDataVariable, MudServerDataValue>> for MudServerDataValue {
    fn from(table: HashMap<MudServerDataVariable, MudServerDataValue>) -> MudServerDataValue {
        MudServerDataValue::Table(table)
    }
}

#[cfg(test)]
mod tests {
    use super::{consts, encoded_msdp_length, MudServerData, MudServerDataValue};
    use bytes::BytesMut;
    use std::collections::HashMap;

    #[test]
    fn test_encode() {
        let mut map = HashMap::new();
        map.insert("LIST".to_string(), MudServerDataValue::from("COMMANDS"));


        // MSDP_VAR "LIST" MSDP_VAL "COMMANDS"
        let expected = 14;
        let result = encoded_msdp_length(&MudServerData::from(map));
        assert_eq!(expected, result, "encoded size didn't match");
    }
}