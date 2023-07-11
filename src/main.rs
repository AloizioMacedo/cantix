use std::{collections::HashMap, env};

use ::serenity::prelude::GatewayIntents;
use chatgpt::prelude::ChatGPT;
use indicium::simple::SearchIndex;
use serde::{Deserialize, Serialize};
use shuttle_poise::ShuttlePoise;
use shuttle_runtime::Context as _;
use shuttle_secrets::SecretStore;

struct Data {
    search_index: SearchIndex<u8>,
    heroes: HashMap<u8, String>,
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
#[allow(non_snake_case)]
struct DataGL {
    heroStats: HeroStats,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct HeroStats {
    winWeek: Vec<HeroWinCount>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct HeroWinCount {
    winCount: f64,
    matchCount: f64,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct ConstantGL {
    constants: ConstantQuery,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct ConstantQuery {
    hero: HeroConstantQuery,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct HeroConstantQuery {
    stats: HeroData,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct HeroData {
    attackType: String,
    startingArmor: f32,
    startingDamageMin: f32,
    startingDamageMax: f32,
    attackRate: f32,
    attackRange: f32,
    primaryAttribute: String,
    strengthBase: u16,
    strengthGain: f32,
    intelligenceBase: u16,
    intelligenceGain: f32,
    agilityBase: u16,
    agilityGain: f32,
    hpRegen: f32,
    mpRegen: f32,
    moveSpeed: f32,
    moveTurnRate: f32,
}

/// Get hero static data.
#[poise::command(slash_command)]
async fn get_hero(ctx: Context<'_>, name: String) -> Result<(), Error> {
    dotenvy::from_filename("Secrets.toml").unwrap();
    let endpoint = "https://api.stratz.com/graphql";

    let query = r#"query myQuery($id: Short!) {
            constants {
              hero(id: $id) {
                stats {
                  attackType
                  startingArmor
                  startingDamageMin
                  startingDamageMax
                  attackRate
                  attackRange
                  primaryAttribute
                  strengthBase
                  strengthGain
                  intelligenceBase
                  intelligenceGain
                  agilityBase
                  agilityGain
                  hpRegen
                  mpRegen
                  moveSpeed
                  moveTurnRate
                }
              }
            }
          }"#;

    let headers: HashMap<&str, String> = [(
        "authorization",
        format!("Bearer {}", env::var("DOTA_API").unwrap()),
    )]
    .into();

    let client = gql_client::Client::new_with_headers(endpoint, headers);

    let search_index = &ctx.data().search_index;
    let heroes = &ctx.data().heroes;

    let id = if let Some(hero_id) = search_index.search(&name).first() {
        **hero_id
    } else {
        ctx.say("Hero not found!").await?;
        return Ok(());
    };

    let data = client
        .query_with_vars::<ConstantGL, Var>(query, Var { id })
        .await;

    let data = data.unwrap().unwrap();
    let hero_data = data.constants.hero.stats;

    ctx.say(format!(
        "> ## {}
        > ### Health, Mana and Armor
        > Health: {}  +{:.1}
        > Mana: {}  +{:.1}
        > Armor: {:.1}
        > ### Stats:
        > Primary: {}
        > Str: {}  +{:.1}
        > Agi: {}  +{:.1}
        > Int: {}  +{:.1}
        > ### Attack:
        > {}; Range: {};
        > Attack Rate: {:.1}
        > Dmg min-max: {}-{}
        > ### Movement:
        > Speed: {}
        > Turn Rate: {}
        ",
        heroes.get(&id).unwrap(),
        120 + 22 * hero_data.strengthBase,
        hero_data.hpRegen,
        75 + 12 * hero_data.intelligenceBase,
        hero_data.mpRegen,
        hero_data.startingArmor,
        hero_data.primaryAttribute,
        hero_data.strengthBase,
        hero_data.strengthGain,
        hero_data.agilityBase,
        hero_data.agilityGain,
        hero_data.intelligenceBase,
        hero_data.intelligenceGain,
        hero_data.attackType,
        hero_data.attackRange,
        hero_data.attackRate,
        hero_data.startingDamageMin,
        hero_data.startingDamageMax,
        hero_data.moveSpeed,
        hero_data.moveTurnRate
    ))
    .await?;

    // ctx.send(|m| {
    //     m.content("Click the button below to open the modal")
    //         .components(|c| {
    //             c.create_action_row(|a| {
    //                 a.create_button(|b| {
    //                     b.custom_id("open_modal")
    //                         .label("Open modal")
    //                         .style(poise::serenity_prelude::ButtonStyle::Success)
    //                 })
    //             })
    //         })
    // })
    // .await?;

    Ok(())
}

/// Get hero winrate in the last 4 weeks.
#[poise::command(slash_command)]
async fn get_winrate(ctx: Context<'_>, name: String) -> Result<(), Error> {
    dotenvy::from_filename("Secrets.toml").unwrap();
    let endpoint = "https://api.stratz.com/graphql";

    let query =
        r#"query myQuery($id: Short) {heroStats {winWeek(heroIds: [$id]) {winCount matchCount} }}"#;

    let headers: HashMap<&str, String> = [(
        "authorization",
        format!("Bearer {}", env::var("DOTA_API").unwrap()),
    )]
    .into();
    let client = gql_client::Client::new_with_headers(endpoint, headers);

    let search_index = &ctx.data().search_index;
    let heroes = &ctx.data().heroes;

    let id = if let Some(hero_id) = search_index.search(&name).first() {
        **hero_id
    } else {
        ctx.say("Hero not found!").await?;
        return Ok(());
    };

    let data = client
        .query_with_vars::<DataGL, Var>(query, Var { id })
        .await;

    let data = data.unwrap().unwrap();

    let wins_total: f64 = data
        .heroStats
        .winWeek
        .iter()
        .take(4)
        .map(|x| x.winCount)
        .sum();
    let total: f64 = data
        .heroStats
        .winWeek
        .iter()
        .take(4)
        .map(|x| x.matchCount)
        .sum();

    ctx.say(format!(
        "> {} has {:.2}% winrate this month.",
        heroes.get(&id).expect(
            r#"The result from the search should
            be an index in the heroes hashmap."#
        ),
        100. * wins_total / total
    ))
    .await?;

    Ok(())
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let heroes: Vec<HeroDataOpenAi> = reqwest::get("https://api.opendota.com/api/heroes")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let heroes: HashMap<u8, String> = heroes
        .into_iter()
        .map(|x| (x.id, x.localized_name))
        .collect();

    let mut search_index: SearchIndex<u8> = SearchIndex::default();

    heroes
        .iter()
        .for_each(|(key, value)| search_index.insert(key, value));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![hello(), get_winrate(), get_hero()],
            ..Default::default()
        })
        .token(&discord_token)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    search_index,
                    heroes,
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}

#[derive(Deserialize, Debug)]
struct HeroDataOpenAi {
    id: u8,
    localized_name: String,
}

#[derive(Serialize)]
struct Var {
    id: u8,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use indicium::simple::SearchIndex;

    use crate::HeroDataOpenAi;

    #[tokio::test]
    async fn get_hero_data() {
        let heroes: Vec<HeroDataOpenAi> = reqwest::get("https://api.opendota.com/api/heroes")
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let heroes: HashMap<u8, String> = heroes
            .into_iter()
            .map(|x| (x.id, x.localized_name))
            .collect();

        let mut search_index: SearchIndex<u8> = SearchIndex::default();

        heroes
            .iter()
            .for_each(|(key, value)| search_index.insert(key, value));

        println!("{:?}", heroes);
        let h = search_index.search("anti-mage");

        println!("{:?}", h);
    }

    #[tokio::test]
    async fn it_works() {
        // dotenvy::from_filename("Secrets.toml").unwrap();
        // let endpoint = "https://api.stratz.com/graphql";

        // let x = r#"query myQuery($id: Short) {heroStats {winMonth(heroIds: [$id]) {heroId winCount matchCount} }}"#;

        // println!("{}", env::var("DOTA_API").unwrap());
        // let headers: HashMap<&str, String> = [(
        //     "authorization",
        //     format!("Bearer {}", env::var("DOTA_API").unwrap()),
        // )]
        // .into();
        // let client = gql_client::Client::new_with_headers(endpoint, headers);

        // let data = client
        //     .query_with_vars::<DataGL, Var>(x, Var { id: 2 })
        //     .await;

        // println!("{:?}", data.as_ref().err());
        // println!("{:?}", data);
    }
}
