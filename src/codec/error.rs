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

///
/// Decoding Errors
///
#[derive(Debug)]
pub enum TelnetError {
    IOError(std::io::Error),
    /// An error occurred while encoding.
    EncodingError(String),
    /// An error occurred while decoding.
    DecodingError(String),
    /// An unknown or invalid command was used
    UnknownCommand(u8),
}

impl std::error::Error for TelnetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TelnetError::IOError(inner) => Some(inner),
            TelnetError::EncodingError(_) => None,
            TelnetError::DecodingError(_) => None,
            TelnetError::UnknownCommand(_) => None,
        }
    }
}

impl std::fmt::Display for TelnetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelnetError::IOError(inner) => {
                write!(f, "TelnetError::IOError({})", inner)
            }
            TelnetError::EncodingError(error) => {
                write!(f, "TelnetError::EncodingError({})", error)
            }
            TelnetError::DecodingError(error) => {
                write!(f, "TelnetError::DecodingError({})", error)
            }
            TelnetError::UnknownCommand(cmd) => {
                write!(f, "TelnetError::UnknownCommand({:#X})", cmd)
            }
        }
    }
}

impl From<std::io::Error> for TelnetError {
    fn from(error: std::io::Error) -> TelnetError {
        TelnetError::IOError(error)
    }
}
