mod error;
mod menu_page;
pub use error::Error;
mod location_page;
mod remove_excess_whitespace;
mod static_selector;
mod text_from_selection;

pub use location_page::LocationMeta;
pub use location_page::Locations;
pub use remove_excess_whitespace::remove_excess_whitespace;
