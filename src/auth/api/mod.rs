// This module contains database CRUD operations for each of the models.
// It also contains helpful higher-level functions for common tasks like "logging in", "send verification email", etc

pub mod account_api;
pub mod session_api;
pub mod user_api;

use crate::schema;

use super::models::account_model;
use super::models::session_model;
use super::models::user_model;