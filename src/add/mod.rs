mod database;

use anyhow::Result;

trait AddFeature {
    fn add_feature(&self) -> Result<()>;
}
