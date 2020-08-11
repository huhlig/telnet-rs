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
use futures::task::{Context, Poll};
use futures::{AsyncRead, AsyncWrite, Sink, Stream};
use std::pin::Pin;
use tracing::{event, Level};

///
///
///
pub struct TelnetCodec<S>
where
    S: AsyncRead + AsyncWrite,
{
    stream: S,
    encoder_buffer: BytesMut,
    decoder_buffer: BytesMut,
    decoder_state: DecoderState,
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

impl<S> TelnetCodec<S>
where
    S: AsyncRead + AsyncWrite,
{
    /// Create new Telnet Codec
    pub fn new(stream: S) -> Self {
        let encoder_buffer = BytesMut::with_capacity(4096);
        let decoder_buffer = BytesMut::with_capacity(4096);
        let decoder_state = DecoderState::NormalData;
        TelnetCodec {
            stream,
            encoder_buffer,
            decoder_buffer,
            decoder_state,
        }
    }
    ///
    /// Decode incoming data frame by frame
    ///
    pub fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<TelnetFrame>, TelnetError> {
        while src.remaining() > 0 {
            let byte = src.get_u8();
            match (self.decoder_state, byte) {
                (DecoderState::NormalData, consts::IAC) => {
                    self.decoder_state = DecoderState::InterpretAsCommand;
                }
                (DecoderState::NormalData, _) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Data(byte)));
                }
                (DecoderState::InterpretAsCommand, consts::NOP) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::NoOperation));
                }
                (DecoderState::InterpretAsCommand, consts::DM) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::DataMark));
                }
                (DecoderState::InterpretAsCommand, consts::BRK) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Break));
                }
                (DecoderState::InterpretAsCommand, consts::IP) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::InterruptProcess));
                }
                (DecoderState::InterpretAsCommand, consts::AO) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::AbortOutput));
                }
                (DecoderState::InterpretAsCommand, consts::AYT) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::AreYouThere));
                }
                (DecoderState::InterpretAsCommand, consts::EC) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::EraseCharacter));
                }
                (DecoderState::InterpretAsCommand, consts::EL) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::EraseLine));
                }
                (DecoderState::InterpretAsCommand, consts::GA) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::GoAhead));
                }
                (DecoderState::InterpretAsCommand, consts::IAC) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Data(consts::IAC)));
                }
                (DecoderState::InterpretAsCommand, consts::DO) => {
                    self.decoder_state = DecoderState::NegotiateDo;
                }
                (DecoderState::InterpretAsCommand, consts::DONT) => {
                    self.decoder_state = DecoderState::NegotiateDont;
                }
                (DecoderState::InterpretAsCommand, consts::WILL) => {
                    self.decoder_state = DecoderState::NegotiateWill;
                }
                (DecoderState::InterpretAsCommand, consts::WONT) => {
                    self.decoder_state = DecoderState::NegotiateWont;
                }
                (DecoderState::InterpretAsCommand, consts::SB) => {
                    self.decoder_state = DecoderState::Subnegotiate;
                }
                (DecoderState::InterpretAsCommand, _) => {
                    // Return to NormalData State, and return a No Operation
                    event!(Level::WARN, "Received Unknown Command {:#X}", byte);
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::NoOperation));
                }
                (DecoderState::NegotiateDo, _) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Do(TelnetOption::from(byte))));
                }
                (DecoderState::NegotiateDont, _) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Dont(TelnetOption::from(
                        byte,
                    ))));
                }
                (DecoderState::NegotiateWill, _) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Will(TelnetOption::from(
                        byte,
                    ))));
                }
                (DecoderState::NegotiateWont, _) => {
                    self.decoder_state = DecoderState::NormalData;
                    return Ok(Some(TelnetFrame::Wont(TelnetOption::from(
                        byte,
                    ))));
                }
                (DecoderState::Subnegotiate, _) => {
                    self.decoder_state = DecoderState::SubnegotiateArgument(
                        TelnetOption::from(byte),
                    );
                }
                (DecoderState::SubnegotiateArgument(option), consts::IAC) => {
                    self.decoder_state =
                        DecoderState::SubnegotiateArgumentIAC(option);
                }
                (DecoderState::SubnegotiateArgument(_option), _) => {
                    self.decoder_buffer.put_u8(byte);
                }
                (
                    DecoderState::SubnegotiateArgumentIAC(option),
                    consts::IAC,
                ) => {
                    self.decoder_state =
                        DecoderState::SubnegotiateArgument(option);
                    self.decoder_buffer.put_u8(consts::IAC);
                }
                (DecoderState::SubnegotiateArgumentIAC(option), consts::SE) => {
                    self.decoder_state = DecoderState::NormalData;
                    let buffer = Vec::from(self.decoder_buffer.as_ref());
                    self.decoder_buffer.clear();
                    let argument = match option {
                        TelnetOption::MSDP => decode_msdp(buffer),
                        _ => TelnetArgument::Unknown(buffer),
                    };
                    return Ok(Some(TelnetFrame::Subnegotiate(
                        option, argument,
                    )));
                }
                (DecoderState::SubnegotiateArgumentIAC(_), _) => {
                    // TODO: Evaluate if better to return back to SubnegotiateArgumentIAC state and keep buffer
                    self.decoder_state = DecoderState::NormalData;
                    self.decoder_buffer.clear();
                    event!(Level::WARN, "Received Unknown or invalid Command during Subnegotiation {:#X}. Aborting", byte);
                    return Ok(Some(TelnetFrame::NoOperation));
                }
            }
        }
        Ok(None)
    }

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
                dst.reserve(5 + argument.len());
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::SB);
                dst.put_u8(option.into());
                dst.put_slice(&arguments);
                dst.put_u8(consts::IAC);
                dst.put_u8(consts::SE);
            }
        }
        Ok(())
    }
}

impl<S> Stream for TelnetCodec<S>
where
    S: AsyncRead + AsyncWrite,
{
    type Item = TelnetFrame;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

impl<S> Sink<TelnetFrame> for TelnetCodec<S>
where
    S: AsyncRead + AsyncWrite,
{
    type Error = TelnetError;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: TelnetFrame,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }
}

fn decode_msdp(buffer: Vec<u8>) -> TelnetArgument {
    TelnetArgument::Unknown(buffer)
}
