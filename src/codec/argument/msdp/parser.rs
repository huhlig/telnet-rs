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

use bytes::Buf;
use nom::{IResult};
use super::{consts, MudServerData};
use crate::codec::TelnetError;

pub fn decode_msdp<T: Buf>(input: &[u8]) -> Result<MudServerData, TelnetError> {
    parser::msdp
}

mod parser {
    use super::consts;
    use crate::codec::argument::msdp::{MudServerDataValue, MudServerData};
    use nom::combinator::{not, map};
    use nom::sequence::{delimited, preceded, pair};
    use nom::bytes::complete::{tag, take_until, take_while};
    use nom::multi::many0;
    use nom::branch::alt;
    use nom::IResult;
    use crate::codec::TelnetError;
    use nom::lib::std::collections::HashMap;

    pub fn msdp(i: &[u8]) -> IResult<&[u8], MudServerData> {
        map(many0(table_entry)(i), MudServerData)
    }

    pub fn variable(i: &[u8]) -> nom::IResult<&[u8], &str> {
        preceded(
            tag(consts::option::msdp::VAR),
            take_until(not(tag(control_byte))),
        )(i)
    }

    pub fn value(i: &[u8]) -> nom::IResult<&[u8], MudServerDataValue> {
        preceded(
            tag(consts::option::msdp::VAL),
            alt((
                map(array, MudServerDataValue::Array),
                map(table, MudServerDataValue::Table),
                map(string, MudServerDataValue::String),
            ))
        )(i)
    }

    pub fn array(i: &[u8]) -> IResult<&[u8], Vec<MudServerDataValue>> {
        delimited(
            tag(consts::option::msdp::ARRAY_OPEN),
            many0(value),
            tag(consts::option::msdp::ARRAY_CLOSE),
        )(i)
    }

    pub fn string(i: &[u8]) -> nom::IResult<&[u8], String> {
        take_while(not(control_byte))
    }

    pub fn table(i: &[u8]) -> nom::IResult<&[u8], Vec<(String,MudServerDataValue)>> {
        delimited(
            tag(consts::option::msdp::TABLE_OPEN),
            many0(table_entry),
            tag(consts::option::msdp::TABLE_CLOSE),
        )
    }

    pub fn table_entry(i: &[u8]) -> nom::IResult<&[u8], (String, MudServerDataValue)> {
        pair(variable, value)
    }

    pub fn control_byte(byte: u8) -> bool {
        (
            consts::NUL,
            consts::IAC,
            consts::option::msdp::VAL,
            consts::option::msdp::VAR,
            consts::option::msdp::TABLE_OPEN,
            consts::option::msdp::TABLE_CLOSE,
            consts::option::msdp::ARRAY_OPEN,
            consts::option::msdp::ARRAY_CLOSE,
        ).contains(byte)
    }
}