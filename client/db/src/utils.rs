
use std::sync::Arc;
use crate::{Database, DbHash, DatabaseSettings, DatabaseSettingsSrc};

pub fn open_database(
	config: &DatabaseSettings,
) -> Result<Arc<dyn Database<DbHash>>, String> {
	let db: Arc<dyn Database<DbHash>> = match &config.source {
		DatabaseSettingsSrc::RocksDb { path, cache_size: _ } => {
			let db_config = kvdb_rocksdb::DatabaseConfig::with_columns(crate::columns::NUM_COLUMNS);
			let path = path.to_str()
				.ok_or_else(|| "Invalid database path".to_string())?;

			let db = kvdb_rocksdb::Database::open(&db_config, &path)
				.map_err(|err| format!("{}", err))?;
			sp_database::as_database(db)
		}
	};

	Ok(db)
}
