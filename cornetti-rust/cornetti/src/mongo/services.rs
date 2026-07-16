use crate::mongo::confs::MongoDBConfig;

/// MongoDB client and database handle.
///
/// Constructed via `new()` which connects to the configured MongoDB instance.
pub struct MongoDBService {
    client: mongodb::Client,
    db: mongodb::Database,
}

impl MongoDBService {
    /// Connects to MongoDB using the given configuration.
    ///
    /// # Errors
    ///
    /// Returns a `mongodb::error::Error` if the connection URI is invalid or
    /// the server is unreachable.
    pub async fn new(config: &MongoDBConfig) -> mongodb::error::Result<Self> {
        let mongo_uri = if config.db_username.is_none() || config.db_password.is_none() {
            format!(
                "mongodb://{}:{}/{}",
                config.db_host, config.db_port, config.db_name,
            )
        } else {
            format!(
                "mongodb://{}:{}@{}:{}/{}?authSource={}&authMechanism={}",
                config.db_username.clone().unwrap(),
                config.db_password.clone().unwrap(),
                config.db_host,
                config.db_port,
                config.db_name,
                config.auth_source,
                config.auth_mechanism
            )
        };

        let options: mongodb::options::ClientOptions =
            mongodb::options::ClientOptions::parse(&mongo_uri).await?;
        let client: mongodb::Client = mongodb::Client::with_options(options)?;
        let db = client.database(&config.db_name);
        Ok(MongoDBService { client, db })
    }

    /// Returns a reference to the MongoDB database.
    pub fn db(&self) -> &mongodb::Database {
        &self.db
    }

    /// Returns a reference to the MongoDB client.
    pub fn client(&self) -> &mongodb::Client {
        &self.client
    }
}
