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

use bytes::BufMut;
use super::{consts, MudServerData, MudServerDataValue};
use std::collections::HashMap;

/// Get Encoded Length of `MudServerData`
pub fn encoded_msdp_length(msdp: &MudServerData) -> usize {
    msdp.0.iter()
        .map(|(key, value)|
            encoded_msdp_key_length(key) + encoded_msdp_value_length(value)
        )
        .sum()
}

fn encoded_msdp_key_length(key: &str) -> usize {
    1usize + key.len()
}

fn encoded_msdp_value_length(value: &MudServerDataValue) -> usize {
    1usize + match value {
        MudServerDataValue::String(string) => encoded_msdp_string_length(string),
        MudServerDataValue::Array(array) => encoded_msdp_array_length(array),
        MudServerDataValue::Table(table) => encoded_msdp_table_length(table)
    }
}

fn encoded_msdp_string_length(string: &str) -> usize {
    string.len()
}

fn encoded_msdp_array_length(array: &Vec<MudServerDataValue>) -> usize {
    2 + array.iter().map(|value| encoded_msdp_value_length(value)).sum::<usize>()
}

fn encoded_msdp_table_length(table: &HashMap<String, MudServerDataValue>) -> usize {
    2 + table.iter()
        .map(|(key, value)|
            encoded_msdp_key_length(key) + encoded_msdp_value_length(value)
        ).sum::<usize>()
}

pub fn encode_msdp<T: BufMut>(msdp: &MudServerData, dst: &mut T) {
    for (key, value) in &msdp.0 {
        encode_msdp_key(key, dst);
        encode_msdp_value(value, dst);
    }
}


fn encode_msdp_key<T: BufMut>(key: &str, dst: &mut T) {
    dst.put_u8(consts::option::msdp::VAR);
    dst.put(key.as_bytes());
}

fn encode_msdp_value<T: BufMut>(value: &MudServerDataValue, dst: &mut T) {
    dst.put_u8(consts::option::msdp::VAL);
    match value {
        MudServerDataValue::String(str) => encode_msdp_string(str, dst),
        MudServerDataValue::Array(array) => encode_msdp_array(array, dst),
        MudServerDataValue::Table(table) => encode_msdp_table(table, dst),
    }
}

fn encode_msdp_string<T: BufMut>(string: &str, dst: &mut T) {
    dst.put(string.as_bytes());
}

fn encode_msdp_array<T: BufMut>(array: &Vec<MudServerDataValue>, dst: &mut T) {
    dst.put_u8(consts::option::msdp::ARRAY_OPEN);
    for value in array {
        dst.put_u8(consts::option::msdp::VAL);
        encode_msdp_value(value, dst);
    }
    dst.put_u8(consts::option::msdp::ARRAY_CLOSE);
}

fn encode_msdp_table<T: BufMut>(table: &HashMap<String, MudServerDataValue>, dst: &mut T) {
    dst.put_u8(consts::option::msdp::TABLE_OPEN);
    for (key, value) in table {
        encode_msdp_key(key, dst);
        encode_msdp_value(value, dst);
    }
    dst.put_u8(consts::option::msdp::TABLE_CLOSE);
}

