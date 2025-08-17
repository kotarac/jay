//! Tools for configuring built-in bar.

use serde::{Deserialize, Serialize};

/// Where the built-in bar is displayed.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum BarLocation {
    /// Displays the built-in bar at the top.
    Top,
    /// Displays the built-in bar at the bottom.
    Bottom,
}

/// Sets where the built-in bar is displayed.
///
/// The default is `BarLocation::Top`.
pub fn set_bar_location(location: BarLocation) {
    get!().set_bar_location(location);
}
