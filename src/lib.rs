pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod ical;
pub mod mcp;
pub mod models;
pub mod services;
pub mod state;
pub mod templates;

pub use error::{AppError, AppResult};
pub use state::AppState;
