use std::collections::HashMap;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder, ImageSource};
use crate::birthday::BirthdayPerson;
use crate::config::{ConfigFile, PersonDiscordConfig};

const ASSET_URL_BASE: &'static str = "https://morgverd.com/assets/images/birthdays";
const RANDOM_MESSAGES: [&'static str; 3] = [
    "Can you believe its been a year already?",
    "The grow up so fast.",
    "As is tradition around here,"
];

#[derive(Serialize, Debug)]
struct DiscordWebhook {
    username: String,
    avatar_url: String,
    content: String,
    embeds: Vec<Embed>
}

fn create_embed(people: Vec<(String, &PersonDiscordConfig)>) -> Embed {

    // Get all mentions either as a discord mention or just bold name if there isn't an ID.
    let mut mentions = Vec::<String>::with_capacity(people.len());
    for (name, config) in people {
        mentions.push(
            if let Some(id) = &config.id {
                format!("<@{id}>")
            } else {
                format!("**{name}**")
            }
        );
    }

    // Choose a random prefix message.
    let mut rng = thread_rng();
    let random_prefix = match RANDOM_MESSAGES.choose(&mut rng) {
        None => "Happy birthday to you!",
        Some(r) => r
    };

    // Build the final embed.
    EmbedBuilder::new()
        .title("Happy Birthday!")
        .description(format!("{} Please wish a very happy birthday to {}.", random_prefix, mentions.join(", ")))
        .image(ImageSource::url(format!("{ASSET_URL_BASE}/happy_birthday.gif")).unwrap())
        .footer(EmbedFooterBuilder::new("Sent by morgverd.com birthdays manager v2."))
        .build()
}

pub fn run(config: &ConfigFile, people: Vec<&BirthdayPerson>) -> () {

    // Split each person into a map of the discord servers they belong to.
    let mut server_members = HashMap::<String, Vec<(String, &PersonDiscordConfig)>>::new();
    let mut server_pings = HashMap::<String, bool>::new();

    for person in people {

        // Skip if the person doesn't have any discord account associated.
        if let Some(discord) = &person.discord {
            for server in discord.servers.iter() {

                // Get the default server ping status.
                if !server_pings.contains_key(server) {

                    // Make sure the server ID actually exists.
                    let server_config = config.servers.get(server);
                    if server_config.is_none() {
                        eprintln!("BirthdayPerson '{}' has invalid discord server ID '{}'", person.name, server);
                        continue;
                    }
                    server_pings.insert(server.clone(), server_config.unwrap().default_ping_everyone);
                }

                // Add the person as a server member and keep track of the ping state.
                server_members.entry(server.clone()).or_insert_with(Vec::new).push((person.name.clone(), discord));
                if let Some(ping) = discord.ping_everyone {
                    server_pings.entry(server.clone()).and_modify(|v| { *v = ping; });
                }
            }
        }
    }
    if server_members.is_empty() {
        return;
    }

    // Send webhook to each target server.
    for (server_id, people) in server_members {

        // Create webhook with an embed.
        let webhook = DiscordWebhook {
            username: "The Birthday Bot".to_owned(),
            avatar_url: format!("{ASSET_URL_BASE}/birthday_cake.png"),
            content: (if *server_pings.get(&server_id).unwrap_or(&false) { "@everyone" } else { "" }).to_owned(),
            embeds: vec![create_embed(people)],
        };

        // Serialize the webhook struct into JSON to send.
        let json = match serde_json::to_string::<DiscordWebhook>(&webhook) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Couldn't serialize DiscordWebhook for server '{server_id}': {e:#?}!");
                continue;
            }
        };

        // Post the webhook.
        match minreq::post(&config.servers.get(&server_id).unwrap().webhook)
            .with_body(json)
            .with_header("Content-Type", "application/json")
            .send()
        {
            Ok(response) => println!(
                "Webhook response {server_id}: {}",
                if response.status_code == 204 { "Successfully sent webhook!" } else { "Failed to send webhook!" }
            ),
            Err(error) => {
                println!("Webhook error {server_id}: {error:#?}");
            }
        }
    }
}