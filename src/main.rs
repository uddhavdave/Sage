mod commands;
mod evt_handler;

use commands::api_calls::qod_api;
use commands::general::HELP;
use commands::ADMIN_GROUP;
use commands::EMOJI_GROUP;
use commands::GENERAL_GROUP;
use evt_handler::Handler;

use db_handler::Database;

use serenity::{client::Client, http::Http};
use serenity::{framework::standard::StandardFramework, model::id::ChannelId};
use std::collections::HashSet;
use std::env;
use std::u64;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let api_token = env::var("API_TOKEN").unwrap();
    let mut client: Client;
    match &api_token.len() {
        0 => panic!("Token Not found!!"),
        _ => {
            let http = Http::new_with_token(&api_token);
            let (owners, _) = match http.get_current_application_info().await {
                Ok(info) => {
                    let mut owners = HashSet::new();
                    if let Some(team) = info.team {
                        owners.insert(team.owner_user_id);
                    } else {
                        owners.insert(info.owner.id);
                    }
                    match http.get_current_user().await {
                        Ok(bot_id) => (owners, bot_id.id),
                        Err(why) => panic!("Could not access the bot id: {:?}", why),
                    }
                }
                Err(why) => panic!("Could not access application info: {:?}", why),
            };
            client = Client::builder(&api_token)
                .event_handler(Handler)
                .framework(
                    StandardFramework::new()
                        .configure(|c| c.prefix("!").owners(owners))
                        .help(&HELP)
                        .group(&GENERAL_GROUP)
                        .group(&EMOJI_GROUP)
                        .group(&ADMIN_GROUP),
                )
                .await
                .unwrap();
        }
    };

    let client_ch = client.cache_and_http.clone();

    let mut db = Database::new();
    let db_uri = env::var("MONGO_URI").unwrap();
    let _ = db.make_connection(db_uri).await;

    tokio::spawn(async move {
        loop {
            if let Some(data) = db.get_cached_data().await {
                let subscriber = data.as_ref();
                println!("{:?}", subscriber.funny);
                for channels in &subscriber.funny {
                    match qod_api::quote_of_the_day("funny").await {
                        Ok(qod_tuple) => match channels.len() {
                            0 => println!("Empty Entry"),
                            _ => {
                                let (quote, author) = *qod_tuple;
                                let message = String::from(format!("{}\n-_{}_", quote, author));
                                let chid: u64 = channels.parse::<u64>().expect("Not a u64 number");
                                let channel = ChannelId(chid);
                                channel
                                    .say(&client_ch.http, &message)
                                    .await
                                    .expect("Failed to deliver message");
                                println!("Sent quote to {}", chid);
                            }
                        },
                        Err(why) => println!("Error occurred: {}", why),
                    };
                }
            }
            sleep(Duration::from_secs(86400)).await; //86400 seconds in a day
        }
    });

    //Send client to quotes_task
    if let Err(msg) = client.start().await {
        println!("Error : {:?}", msg);
    }
}
