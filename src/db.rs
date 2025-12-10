use mongodb::{options::ClientOptions, Client, Database};

pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub fn init() -> Self {


        let mongo_uri = "mongodb://localhost:27017";
        println!("Using Mongo URI: {}", mongo_uri);

        let mut opts = ClientOptions::parse(&mongo_uri)
            .expect("Failed to parse Mongo URI");

        opts.app_name = Some("MyRocketApp".to_string());

        let client = Client::with_options(opts).expect("Failed to initialize Mongo client");

        let db_name = "391_FinalProject";
        let db = client.database(&db_name);

        MongoRepo { db }
    }
}
