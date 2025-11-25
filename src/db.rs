use mongodb::{options::ClientOptions, Client, Database};

pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let mut opts = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .expect("Failed to parse DB URL");

        opts.app_name = Some("MyRocketApp".to_string());
        let client = Client::with_options(opts).expect("Failed to init client");

        let db = client.database("391_FinalProject");
        MongoRepo { db }
    }
}