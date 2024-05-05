pub mod diesel;
pub mod sqlx;

use anyhow::Result;
use std::path::Path;

trait UpdateDatabaseFiles {
    fn update_config(&self, path: &Path) -> Result<()>;
    fn update_startup(&self, path: &Path) -> Result<()>;
}
