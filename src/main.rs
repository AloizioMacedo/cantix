use ::serenity::prelude::GatewayIntents;
use chatgpt::prelude::ChatGPT;
use shuttle_poise::ShuttlePoise;
use shuttle_runtime::Context as _;
use shuttle_secrets::SecretStore;

struct Data {} // User data, which is stored and accessible in all command invocations
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

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello(), ask_gpt()],
            ..Default::default()
        })
        .token(discord_token)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
