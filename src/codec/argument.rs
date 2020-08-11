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

mod msdp;
mod mssp;
mod naocrd;
mod naohts;
mod status;

use super::{consts, TelnetError};
use bytes::BufMut;
use crate::codec::argument::msdp::{encoded_msdp_length, encode_msdp};

/// Telnet Subnegotiation Argument
#[derive(Clone, Debug, PartialEq)]
pub enum TelnetArgument {
    MSDP(msdp::MudServerData),
    MSSP(mssp::MudServerStatus),
    NAOCRD(naocrd::NAOCRD),
    NAOHTS(naohts::NAOHTS),
    Status(status::OptionStatus),
    Unknown(Vec<u8>),
}


pub fn encoded_argument_length(argument: &TelnetArgument) -> usize {
    match argument {
        TelnetArgument::MSDP(msdp) => encoded_msdp_length(msdp),
        _ => 0,
    }
}

pub fn encode_argument<T: BufMut>(argument: &TelnetArgument, dst: &mut T) {
    match argument {
        TelnetArgument::MSDP(msdp) => encode_msdp(msdp, dst),
        _ => unimplemented!(),
    }
}
