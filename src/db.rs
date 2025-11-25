use std::env;
use mongodb::{options::ClientOptions, Client, Database};
use dotenvy::dotenv;

pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();

        let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set in .env");
        println!("Using Mongo URI: {}", mongo_uri);


        let mut opts = ClientOptions::parse(&mongo_uri)
            .await
            .expect("Failed to parse DB URL");

        opts.app_name = Some("MyRocketApp".to_string());

        let client = Client::with_options(opts).expect("Failed to init client");

        let db_name = env::var("MONGO_DB").unwrap_or_else(|_| "391_FinalProject".to_string());
        let db = client.database(&db_name);

        MongoRepo { db }
    }
}
