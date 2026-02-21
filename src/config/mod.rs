pub mod icons;
pub mod loader;
pub mod types;

pub use icons::resolve_icons;
pub use loader::{find_config, load_config};
pub use types::{FlatEntry, LaunchType};
