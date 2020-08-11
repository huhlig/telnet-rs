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

use super::{consts, TelnetArgument, TelnetError, TelnetFrame, TelnetOption};
use bytes::{Buf, BufMut, BytesMut};
use tracing::{event, Level};

///
///
///
pub struct TelnetDecoder {
    buffer: BytesMut,
    state: DecoderState,
}

#[derive(Clone, Copy, Debug)]
enum DecoderState {
    /// Normal Data
    NormalData,
    /// Received IAC, Next byte is Command
    InterpretAsCommand,
    /// Received Do Command, Next Byte is arguments
    NegotiateDo,
    /// Received Dont Command, Next Byte is arguments
    NegotiateDont,
    /// Received Will Command, Next Byte is arguments
    NegotiateWill,
    /// Received Wont Command, Next Byte is arguments
    NegotiateWont,
    /// Received Subnegotiate Command, Next Byte is arguments
    Subnegotiate,
    /// Received Subnegotiate Option, Next Bytes are arguments
    SubnegotiateArgument(TelnetOption),
    /// Received IAC during Subnegotiation, Next Byte is command
    SubnegotiateArgumentIAC(TelnetOption),
}

impl Default for TelnetDecoder {
    fn default() -> TelnetDecoder {
        TelnetDecoder::new(4096)
    }
}

impl TelnetDecoder {
    /// Create a new TelnetDecoder
    pub fn new(buffer_size: usize) -> Self {
        TelnetDecoder {
            buffer: BytesMut::with_capacity(buffer_size),
            state: DecoderState::NormalData,
        }
    }
    /// Reset Decoder and discard buffer
    pub fn reset(&mut self) {
        self.state = DecoderState::NormalData;
        self.buffer.clear();
    }
    /// Decode incoming bytestream frame by frame
    pub fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<TelnetFrame>, TelnetError> {
        while src.remaining() > 0 {
            let byte = src.get_u8();
            match (self.state, byte) {
                (DecoderState::NormalData, consts::IAC) => {
                    self.state = DecoderState::InterpretAsCommand;
                }
                (DecoderState::NormalData, _) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Data(byte)));
                }
                (DecoderState::InterpretAsCommand, consts::NOP) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::NoOperation));
                }
                (DecoderState::InterpretAsCommand, consts::DM) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::DataMark));
                }
                (DecoderState::InterpretAsCommand, consts::BRK) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Break));
                }
                (DecoderState::InterpretAsCommand, consts::IP) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::InterruptProcess));
                }
                (DecoderState::InterpretAsCommand, consts::AO) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::AbortOutput));
                }
                (DecoderState::InterpretAsCommand, consts::AYT) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::AreYouThere));
                }
                (DecoderState::InterpretAsCommand, consts::EC) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::EraseCharacter));
                }
                (DecoderState::InterpretAsCommand, consts::EL) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::EraseLine));
                }
                (DecoderState::InterpretAsCommand, consts::GA) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::GoAhead));
                }
                (DecoderState::InterpretAsCommand, consts::IAC) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Data(consts::IAC)));
                }
                (DecoderState::InterpretAsCommand, consts::DO) => {
                    self.state = DecoderState::NegotiateDo;
                }
                (DecoderState::InterpretAsCommand, consts::DONT) => {
                    self.state = DecoderState::NegotiateDont;
                }
                (DecoderState::InterpretAsCommand, consts::WILL) => {
                    self.state = DecoderState::NegotiateWill;
                }
                (DecoderState::InterpretAsCommand, consts::WONT) => {
                    self.state = DecoderState::NegotiateWont;
                }
                (DecoderState::InterpretAsCommand, consts::SB) => {
                    self.state = DecoderState::Subnegotiate;
                }
                (DecoderState::InterpretAsCommand, _) => {
                    // Return to NormalData State, and return a No Operation
                    event!(Level::WARN, "Received Unknown Command {:#X}", byte);
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::NoOperation));
                }
                (DecoderState::NegotiateDo, _) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Do(TelnetOption::from(byte))));
                }
                (DecoderState::NegotiateDont, _) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Dont(TelnetOption::from(
                        byte,
                    ))));
                }
                (DecoderState::NegotiateWill, _) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Will(TelnetOption::from(
                        byte,
                    ))));
                }
                (DecoderState::NegotiateWont, _) => {
                    self.state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Wont(TelnetOption::from(
                        byte,
                    ))));
                }
                (DecoderState::Subnegotiate, _) => {
                    self.state = DecoderState::SubnegotiateArgument(
                        TelnetOption::from(byte),
                    );
                }
                (DecoderState::SubnegotiateArgument(option), consts::IAC) => {
                    self.state = DecoderState::SubnegotiateArgumentIAC(option);
                }
                (DecoderState::SubnegotiateArgument(_option), _) => {
                    self.buffer.put_u8(byte);
                }
                (
                    DecoderState::SubnegotiateArgumentIAC(option),
                    consts::IAC,
                ) => {
                    self.state = DecoderState::SubnegotiateArgument(option);
                    self.buffer.put_u8(consts::IAC);
                }
                (DecoderState::SubnegotiateArgumentIAC(option), consts::SE) => {
                    self.state = DecoderState::NormalData;
                    let buffer = Vec::from(self.buffer.as_ref());
                    let argument =match option {
                        _=> TelnetArgument::Unknown(buffer)
                    };
                    self.buffer.clear();
                    return Ok(Some(TelnetFrame::Subnegotiate(
                        option, argument,
                    )));
                }
                (DecoderState::SubnegotiateArgumentIAC(_), _) => {
                    // TODO: Evaluate if better to return back to SubnegotiateArgumentIAC state and keep buffer
                    self.state = DecoderState::NormalData;
                    self.buffer.clear();
                    event!(Level::WARN, "Received Unknown or invalid Command during Subnegotiation {:#X}. Aborting", byte);
                    return Ok(Some(TelnetFrame::NoOperation));
                }
            }
        }
        Ok(None)
    }
}
