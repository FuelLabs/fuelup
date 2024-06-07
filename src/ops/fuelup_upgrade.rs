use super::{fuelup_default, fuelup_self, fuelup_update};
use crate::channel::LATEST;
use anyhow::Result;

pub fn upgrade(force: bool) -> Result<()> {
    // self update
    fuelup_self::self_update(force)?;
    // switch to 'latest' channel.
    fuelup_default::default(Some(LATEST.to_owned()))?;
    // update channel
    fuelup_update::update()?;
    Ok(())
}
