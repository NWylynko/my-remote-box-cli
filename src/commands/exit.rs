use anyhow::Result;

use crate::tmux;

pub fn exit_session() -> Result<()> {
    tmux::detach()
}
