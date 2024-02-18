use amqprs::{
    callbacks::DefaultConnectionCallback,
    channel::{BasicAckArguments, BasicConsumeArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties,
};
use mysql_async::{prelude::*, Pool};
use std::error::Error;
use tokio::sync::Semaphore;
use std::sync::Arc;


struct MyConsumer {
    db_pool: Pool,
    semaphore: Arc<Semaphore>
}

impl AsyncConsumer for MyConsumer {
    fn consume<'life0,'life1,'async_trait>(&'life0 mut self,channel: &'life1 amqprs::channel::Channel,deliver:amqprs::Deliver,_basic_properties:BasicProperties,content:Vec<u8> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = ()> + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait {

        let message = unsafe { std::str::from_utf8_unchecked(&content) }; 
        let params = params! {"content" => message};
        Box::pin(async move {
            let permit = self.semaphore.clone().acquire_owned().await;
            match permit {
                Ok(_) => {
                    let db_pool = self.db_pool.clone();
                    let b = BasicAckArguments::new(deliver.delivery_tag(), false);
                    let rabbit_channel = channel.clone();
                    tokio::spawn(async move {
                        let mut conn = match db_pool.get_conn().await {
                            Ok(conn) => conn,
                            Err(e) => {
                                eprintln!("Failed to get DB connection: {:?}", e);
                                return;
                            }
                        };
                        if let Err(e) = conn.exec_drop("INSERT INTO messages (content) VALUES (:content)", params).await {
                            eprintln!("Failed to insert message into DB: {:?}", e);
                        } else {
                            rabbit_channel.basic_ack(b).await.unwrap();
                        }
                    });
                },
                Err(e) => {
                    eprintln!("Semaphore was closed {:?}", e);
                }
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // load .env file into as enviroment variables
    // _ = dotenvy::dotenv();

    // let amqp_uri = std::env::var("AMQP_URI").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    // let mysql_uri = env::var("MYSQL_URI").unwrap_or_else(|_| "".into());

    // Connect to RabbitMQ
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    )).await.expect("Could not connect to RabbitMQ");

    connection.register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let channel_result = connection.open_channel(None).await;
    let channel = match channel_result {
        Ok(c) => c,
        Err(e) => {
            println!("Channel could not be opened {}", e);
            return Ok(())
        }
    };
    println!("Acquired channel to RabbitMQ");

    let mysql_uri = "mysql://root:example@127.0.0.1:3306/exampledb?pool_min=20&pool_max=150";

    let db_pool = Pool::new(mysql_uri);
    let semaphore = Arc::new(Semaphore::new(120)); // Allow up to 120 concurrent accesses
        let cons = MyConsumer{ db_pool: db_pool, semaphore: semaphore };
        let args = BasicConsumeArguments::new("externally_configured_queue", "")
            .auto_ack(false)
            .exclusive(false)
            .finish();
        match channel.basic_consume(cons, args).await {
            Ok(_) => {},
            Err(e) => { println!("Eroare la consumer {}", e) },
        };
    // Keep the tokio runtime running, replace the following line with your application's logic to keep running
    std::future::pending::<()>().await;

    Ok(())
}

