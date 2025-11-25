use std::env;
use mongodb::{options::ClientOptions, Client, Database};
use dotenvy::dotenv;


pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let mongo_uri = env::var("MONGO_URI")

        let mut opts = ClientOptions::parse(&mongo_uri)
            .await
            .expect("Failed to parse DB URL");

        opts.app_name = Some("MyRocketApp".to_string());
        let client = Client::with_options(opts).expect("Failed to init client");

        let db = client.database("391_FinalProject");
        MongoRepo { db }
    }
}