use std::{collections::HashMap, env, ops::Deref, sync::Arc};

use ::serenity::prelude::GatewayIntents;
use chatgpt::prelude::ChatGPT;
use serde::{Deserialize, Serialize};
use shuttle_poise::ShuttlePoise;
use shuttle_runtime::Context as _;
use shuttle_secrets::SecretStore;
use songbird::{SerenityInit, Songbird, SongbirdKey};

struct Data {
    songbird: Arc<Songbird>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(
    ctx: Context<'_>,
    #[description = "Your name."] name: Option<String>,
) -> Result<(), Error> {
    ctx.say(format!(
        "Hello, {}!",
        name.unwrap_or("stranger".to_string())
    ))
    .await?;
    println!("hello");
    Ok(())
}

/// Ask something to ChatGPT!
#[poise::command(slash_command)]
async fn ask_gpt(ctx: Context<'_>, prompt: String) -> Result<(), Error> {
    dotenvy::from_filename("Secrets.toml").unwrap();

    let client =
        ChatGPT::new(std::env::var("GPT_API_KEY").expect("GPT_API_KEY should be in Secrets.toml"))
            .expect("ChatGPT API key should be valid.");

    let response = client.send_message(&prompt).await?;

    ctx.say(&response.message().content).await?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct DataGL {
    heroStats: HeroStats,
}

#[derive(Deserialize, Debug)]
struct HeroStats {
    winMonth: Vec<HeroWinCount>,
}

#[derive(Deserialize, Debug)]
struct HeroWinCount {
    heroId: u8,
    winCount: f64,
    matchCount: f64,
}

/// Get hero winrate this month by ID.
#[poise::command(slash_command)]
async fn get_winrate(ctx: Context<'_>, id: u8) -> Result<(), Error> {
    dotenvy::from_filename("Secrets.toml").unwrap();
    let endpoint = "https://api.stratz.com/graphql";

    let x = r#"query myQuery($id: Short) {heroStats {winMonth(heroIds: [$id]) {heroId winCount matchCount} }}"#;

    let headers: HashMap<&str, String> = [(
        "authorization",
        format!("Bearer {}", env::var("DOTA_API").unwrap()),
    )]
    .into();
    let client = gql_client::Client::new_with_headers(endpoint, headers);

    let data = client.query_with_vars::<DataGL, Var>(x, Var { id }).await;

    let data = data.unwrap().unwrap();

    let wins_total: f64 = data.heroStats.winMonth.iter().map(|x| x.winCount).sum();
    let total: f64 = data.heroStats.winMonth.iter().map(|x| x.matchCount).sum();

    ctx.say(format!(
        "Hero has {:.2}% winrate this month.",
        100. * wins_total / total
    ))
    .await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn join(ctx: Context<'_>) -> Result<(), Error> {
    // let guild = ctx.guild().unwrap();
    let guild_id = ctx.partial_guild().await.unwrap().id;
    let channel_id = ctx.channel_id();

    // let channel_id = guild
    //     .voice_states
    //     .get(&ctx.author().id)
    //     .and_then(|voice_state| voice_state.channel_id);

    // let connect_to = match channel_id {
    //     Some(channel) => channel,
    //     None => {
    //         return Ok(());
    //     }
    // };

    let manager = &ctx.data().songbird;

    // let _handler = manager.join(guild_id, connect_to).await;
    let _handler = manager.join(guild_id, channel_id).await;

    Ok(())
}

/// Play a song!
#[poise::command(slash_command, guild_only)]
async fn play(ctx: Context<'_>, url: String) -> Result<(), Error> {
    println!("{:?}", ctx.author());
    println!("{:?}", ctx.partial_guild().await);

    let partial_guild_id = ctx.partial_guild().await.unwrap().id;

    // let manager = songbird::get(ctx.serenity_context())
    //     .await
    //     .expect("Songbird Voice client placed in at initialisation.")
    //     .clone();

    let manager = &ctx.data().songbird;

    if let Some(handler_lock) = manager.get(partial_guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                return Ok(());
            }
        };

        handler.play_source(source);
    }

    Ok(())
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello(), get_winrate()],
            ..Default::default()
        })
        .token(&discord_token)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    songbird: Songbird::serenity(),
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}

type Short = String;

#[derive(Serialize)]
struct Var {
    id: u8,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env};

    use serenity::json::Value;

    use crate::{DataGL, Var};

    #[tokio::test]
    async fn it_works() {
        dotenvy::from_filename("Secrets.toml").unwrap();
        let endpoint = "https://api.stratz.com/graphql";

        let x = r#"query myQuery($id: Short) {heroStats {winMonth(heroIds: [$id]) {heroId winCount matchCount} }}"#;

        println!("{}", env::var("DOTA_API").unwrap());
        let headers: HashMap<&str, String> = [(
            "authorization",
            format!("Bearer {}", env::var("DOTA_API").unwrap()),
        )]
        .into();
        let client = gql_client::Client::new_with_headers(endpoint, headers);

        let data = client
            .query_with_vars::<DataGL, Var>(x, Var { id: 2 })
            .await;

        println!("{:?}", data.as_ref().err());
        println!("{:?}", data);
    }
}
