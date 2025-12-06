use std::env;
use dotenvy::dotenv;



use mongodb::sync::{Client, Database};

pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub fn init() -> Self {
        dotenv().ok();

        let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set in .env");
        println!("Using Mongo URI: {}", mongo_uri);

        // Sync client creation
        let client = Client::with_uri_str(&mongo_uri)
            .expect("Failed to create MongoDB client");

        let db_name = env::var("MONGO_DB").unwrap_or_else(|_| "391_FinalProject".to_string());
        let db = client.database(&db_name);

        MongoRepo { db }
    }
}
