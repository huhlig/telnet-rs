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

use super::{consts, TelnetError, TelnetFrame};
use bytes::{BufMut, BytesMut};
use crate::codec::argument::{encoded_argument_length, encode_argument};

///
///
///
pub struct TelnetEncoder;

impl Default for TelnetEncoder {
    fn default() -> TelnetEncoder {
        TelnetEncoder {}
    }
}

impl TelnetEncoder {
    pub fn encode(
        &mut self,
        item: TelnetFrame,
        dst: &mut BytesMut,
    ) -> Result<(), TelnetError> {
        match item {
            TelnetFrame::Data(ch) => {
                dst.reserve(2);
                if ch == consts::IAC {
                    dst.put_u8(consts::IAC);
                }
                dst.put_u8(ch);
            }
            TelnetFrame::NoOperation => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::NOP);
            }
            TelnetFrame::DataMark => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::DM);
            }
            TelnetFrame::Break => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::BRK);
            }
            TelnetFrame::InterruptProcess => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::IP);
            }
            TelnetFrame::AbortOutput => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::AO);
            }
            TelnetFrame::AreYouThere => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::AYT);
            }
            TelnetFrame::EraseCharacter => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::EC);
            }
            TelnetFrame::EraseLine => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::EL);
            }
            TelnetFrame::GoAhead => {
                dst.reserve(2);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::GA);
            }
            TelnetFrame::Do(option) => {
                dst.reserve(3);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::DO);
                dst.put_u8(option.into());
            }
            TelnetFrame::Dont(option) => {
                dst.reserve(3);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::DONT);
                dst.put_u8(option.into());
            }
            TelnetFrame::Will(option) => {
                dst.reserve(3);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::WILL);
                dst.put_u8(option.into());
            }
            TelnetFrame::Wont(option) => {
                dst.reserve(3);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::WONT);
                dst.put_u8(option.into());
            }
            TelnetFrame::Subnegotiate(option, argument) => {
                dst.reserve(5 + encoded_argument_length(&argument));
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::SB);
                dst.put_u8(option.into());
                encode_argument(&argument, dst);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::SE);
            }
        }
        Ok(())
    }
}
