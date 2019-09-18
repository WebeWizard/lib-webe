use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};

#[derive(Debug)]
pub enum DBError {
    PoolError(PoolError),
}

pub fn create_db_pool(
    connect_string: String,
) -> Result<Pool<ConnectionManager<MysqlConnection>>, DBError> {
    let connection_manager = ConnectionManager::new(connect_string.as_str());
    // build the database connection pool
    match Pool::builder().max_size(10).build(connection_manager) {
        Ok(db_pool) => return Ok(db_pool),
        Err(err) => return Err(DBError::PoolError(err)),
    }
}
