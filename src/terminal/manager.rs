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

use crate::codec::{TelnetFrame, TelnetOption};
use crate::terminal::TerminalError;

///
/// Network Virtual Terminal Option Manager
///
pub struct OptionManager {
    options: [(State, State); 255],
}

impl OptionManager {
    pub fn allow(&mut self, endpoint: Endpoint) {}
    pub fn disallow(&mut self, endpoint: Endpoint) {}
    /// Handle Incoming
    #[rustfmt::skip]
    fn handle(
        &mut self,
        source: Endpoint,
        action: Action,
        option: TelnetOption,
    ) -> Result<Option<TelnetFrame>, TerminalError> {
        use self::{Action::*, Endpoint::*, State::*};
        // @formatter:off

        match (self.options[option].0, self.options[option].1, source, action) {
        //  (Local State, Remote State, Source, Action) => {  }
            (      Never,            _,      _,      _) => { Ok(Some(TelnetFrame::Wont(option))) }
            (          _,        Never,      _,      _) => { Ok(Some(TelnetFrame::Dont(option))) }
            (_,_,_) => {panic!("Incomplete")}
        }
        // @formatter:on
    }
}

impl Default for OptionManager {
    fn default() -> OptionManager {
        OptionManager {
            options: [(State::No, State::No); 255],
        }
    }
}

pub enum Endpoint {
    // Local Sent message
    Local,
    // Remote Sent message
    Remote,
}

pub enum Action {
    Do,
    Dont,
    Will,
    Wont,
}

pub enum State {
    Never,
    No,
    WantNo,
    WantNoOpposite,
    Yes,
    WantYes,
    WantYesOpposite,
}
