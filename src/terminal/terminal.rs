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

use super::manager::{Action, Endpoint, OptionManager};
use crate::codec::{
    DecodeError, EncodeError, TelnetCodec, TelnetFrame, TelnetOption,
};
use crate::consts;
use crate::terminal::{TerminalError, TerminalInput, TerminalOutput};

use bytes::{BufMut, BytesMut};
use futures::task::{Context, Poll};
use futures::{AsyncRead, AsyncWrite, Future, Sink, Stream};
use std::pin::Pin;

///
/// Telnet Network Virtual Terminal Endpoint
///
pub struct NetworkVirtualTerminal<S>
where
    S: AsyncWrite + AsyncRead,
{
    /// Telnet Codec
    codec: TelnetCodec<S>,
    /// Terminal "Screen" Buffer
    buffer: BytesMut,
    /// Current Option Negotiation State
    options: OptionManager,
}

impl<S> NetworkVirtualTerminal<S>
where
    S: AsyncRead + AsyncWrite,
{
    ///
    /// Create a new Network Virtual Terminal
    ///
    pub fn new(stream: S) -> NetworkVirtualTerminal<S> {
        let codec = TelnetCodec::new(stream);
        let buffer = BytesMut::with_capacity(4096);
        let options = OptionManager::default();
        NetworkVirtualTerminal {
            codec,
            buffer,
            options,
        }
    }
}

impl<S> Stream for NetworkVirtualTerminal<S>
where
    S: AsyncRead + AsyncWrite,
{
    type Item = TerminalOutput;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            match frame {
                TelnetFrame::Data(ch) => {
                    // TODO: Handle Data Stream
                    self.buffer.put_u8(ch);
                    if self.buffer.len() >= self.buffer.capacity() {
                        let data = Vec::from(self.buffer.as_ref());
                        return Poll::Ready(Some(TerminalOutput::BinaryData(
                            data,
                        )));
                    }
                }
                TelnetFrame::AbortOutput => {
                    return Poll::Ready(Some(TerminalOutput::AbortOutput));
                }
                TelnetFrame::AreYouThere => {
                    return Poll::Ready(Some(TerminalOutput::AreYouThere));
                }
                TelnetFrame::Break => {
                    return Poll::Ready(Some(TerminalOutput::Break));
                }
                TelnetFrame::DataMark => {
                    return Poll::Ready(Some(TerminalOutput::DataMark));
                }
                TelnetFrame::EraseCharacter => {
                    return Poll::Ready(Some(TerminalOutput::EraseCharacter));
                }
                TelnetFrame::EraseLine => {
                    return Poll::Ready(Some(TerminalOutput::EraseLine));
                }
                TelnetFrame::GoAhead => {
                    return Poll::Ready(Some(TerminalOutput::GoAhead));
                }
                TelnetFrame::InterruptProcess => {
                    return Poll::Ready(Some(TerminalOutput::InterruptProcess));
                }
                TelnetFrame::NoOperation => {
                    return Poll::Ready(Some(TerminalOutput::NoOperation));
                }
                TelnetFrame::Do(opt) => {
                    let option = TelnetOption::from(opt);
                    if self.options.0.is_none() {
                        self.options.0 = (option, Side::Local)
                    }
                    self.options[opt]
                }
                TelnetFrame::Dont(opt) => {
                    let option = TelnetOption::from(opt);
                }
                TelnetFrame::Will(opt) => {
                    let option = TelnetOption::from(opt);
                }
                TelnetFrame::Wont(opt) => {
                    let option = TelnetOption::from(opt);
                }
                TelnetFrame::Subnegotiate(opt, arguments) => {
                    let option = TelnetOption::from(opt);
                }
            }
        }
    }
}

impl<S> Sink<TerminalInput> for NetworkVirtualTerminal<S>
where
    S: AsyncRead + AsyncWrite,
{
    type Error = EncodeError;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.framed.poll_ready(cx)
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: TerminalInput,
    ) -> Result<(), Self::Error> {
        match item {
            TerminalInput::NoOperation => {
                self.framed.start_send(TelnetFrame::NoOperation)
            }
            TerminalInput::DataMark => {
                self.framed.start_send(TelnetFrame::DataMark)
            }
            TerminalInput::Break => self.framed.start_send(TelnetFrame::Break),
            TerminalInput::InterruptProcess => {
                self.framed.start_send(TelnetFrame::InterruptProcess)
            }
            TerminalInput::AbortOutput => {
                self.framed.start_send(TelnetFrame::AbortOutput)
            }
            TerminalInput::AreYouThere => {
                self.framed.start_send(TelnetFrame::AreYouThere)
            }
            TerminalInput::EraseCharacter => {
                self.framed.start_send(TelnetFrame::EraseCharacter)
            }
            TerminalInput::EraseLine => {
                self.framed.start_send(TelnetFrame::EraseLine)
            }
            TerminalInput::GoAhead => {
                self.framed.start_send(TelnetFrame::GoAhead)
            }
            TerminalInput::AsciiData(data) => {
                // TODO: Handle Compatability settings
                for byte in data.as_bytes() {
                    self.framed.start_send(TelnetFrame::Data(*byte))
                }
            }
            TerminalInput::BinaryData(data) => {
                // TODO: Handle Compatability settings
                for byte in data.as_bytes() {
                    self.framed.start_send(TelnetFrame::Data(*byte))
                }
            }
            TerminalInput::EnableLocalOption(TelnetOption) => {}
            TerminalInput::DisableLocalOption(TelnetOption) => {}
            TerminalInput::EnableRemoteOption(TelnetOption) => {}
            TerminalInput::DisableRemoteOption(TelnetOption) => {}
        }
        Ok(())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.framed.poll_flush(cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.framed.poll_close(cx)
    }
}

pub struct Compatibility {
    send_unicode_while_normal: bool,
    recv_unicode_while_normal: bool,
}

enum NegotationState {
    Never,
    No,
    WantNo,
    WantNoOpposite,
    Yes,
    WantYes,
    WantYesOpposite,
}
