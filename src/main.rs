use std::collections::HashMap;

use ::serenity::prelude::GatewayIntents;
use cantix::{constanthero, matchup, winrate};
use chatgpt::prelude::ChatGPT;
use indicium::simple::SearchIndex;
use serde::{Deserialize, Serialize};
use shuttle_poise::ShuttlePoise;
use shuttle_runtime::Context as _;
use shuttle_secrets::SecretStore;

struct Data {
    search_index: SearchIndex<u8>,
    heroes: HashMap<u8, String>,
    dota_token: String,
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

async fn query_stratz<T>(
    dota_token: &str,
    name: &str,
    query: &str,
    search_index: &SearchIndex<u8>,
) -> Result<(T, u8), &'static str>
where
    T: for<'de> Deserialize<'de>,
{
    let endpoint = "https://api.stratz.com/graphql";

    let headers: HashMap<&str, String> =
        [("authorization", format!("Bearer {}", dota_token))].into();

    let client = gql_client::Client::new_with_headers(endpoint, headers);

    let id = if let Some(hero_id) = search_index.search(name).first() {
        **hero_id
    } else {
        return Err("Hero not found!");
    };

    let data = client
        .query_with_vars::<T, Var>(query, Var { id })
        .await
        .expect("Should be able to get the query")
        .expect("Query should be non-null.");

    Ok((data, id))
}

/// Get "best with", "best against" and "worst against".
#[poise::command(slash_command)]
async fn get_synergies_and_counters(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let query_advantage = r#"query MyQuery($id: Short) {
        heroStats {
          matchUp(heroId: $id, orderBy: 2) {
            with {
              heroId2
              winsAverage
            }
            vs {
              heroId2
              winsAverage
            }
          }
        }
      }"#;

    let query_disadvantage = r#"query MyQuery($id: Short) {
        heroStats {
          matchUp(heroId: $id, orderBy: 3) {
            with {
              heroId2
              winsAverage
            }
            vs {
              heroId2
              winsAverage
            }
          }
        }
      }"#;

    let (data_advantage, _) = query_stratz::<matchup::HeroQuery>(
        &ctx.data().dota_token,
        &name,
        query_advantage,
        &ctx.data().search_index,
    )
    .await?;

    let (data_disadvantage, id) = query_stratz::<matchup::HeroQuery>(
        &ctx.data().dota_token,
        &name,
        query_disadvantage,
        &ctx.data().search_index,
    )
    .await?;

    let hero = data_advantage.heroStats.matchUp.into_iter().next().unwrap();
    let hero_disadvantage = data_disadvantage
        .heroStats
        .matchUp
        .into_iter()
        .next()
        .unwrap();

    let mut vs = hero.vs.clone();
    let mut with = hero.with;
    let mut counters = hero_disadvantage.vs;

    vs.sort_by(|x, y| x.winsAverage.partial_cmp(&y.winsAverage).unwrap().reverse());
    with.sort_by(|x, y| x.winsAverage.partial_cmp(&y.winsAverage).unwrap().reverse());
    counters.sort_by(|x, y| x.winsAverage.partial_cmp(&y.winsAverage).unwrap());

    let vs = &vs[0..5];
    let with = &with[0..5];
    let counters = &counters[0..5];

    let heroes = &ctx.data().heroes;
    ctx.say(format!(
        "
        > # {}
        > ## Best with
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > ## Best against
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > ## Worst against
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        > {}: {:.1}
        ",
        heroes.get(&id).unwrap(),
        heroes.get(&with[0].heroId2).unwrap(),
        with[0].winsAverage * 100.,
        heroes.get(&with[1].heroId2).unwrap(),
        with[1].winsAverage * 100.,
        heroes.get(&with[2].heroId2).unwrap(),
        with[2].winsAverage * 100.,
        heroes.get(&with[3].heroId2).unwrap(),
        with[3].winsAverage * 100.,
        heroes.get(&with[4].heroId2).unwrap(),
        with[4].winsAverage * 100.,
        heroes.get(&vs[0].heroId2).unwrap(),
        vs[0].winsAverage * 100.,
        heroes.get(&vs[1].heroId2).unwrap(),
        vs[1].winsAverage * 100.,
        heroes.get(&vs[2].heroId2).unwrap(),
        vs[2].winsAverage * 100.,
        heroes.get(&vs[3].heroId2).unwrap(),
        vs[3].winsAverage * 100.,
        heroes.get(&vs[4].heroId2).unwrap(),
        vs[4].winsAverage * 100.,
        heroes.get(&counters[0].heroId2).unwrap(),
        counters[0].winsAverage * 100.,
        heroes.get(&counters[1].heroId2).unwrap(),
        counters[1].winsAverage * 100.,
        heroes.get(&counters[2].heroId2).unwrap(),
        counters[2].winsAverage * 100.,
        heroes.get(&counters[3].heroId2).unwrap(),
        counters[3].winsAverage * 100.,
        heroes.get(&counters[4].heroId2).unwrap(),
        counters[4].winsAverage * 100.,
    ))
    .await?;

    Ok(())
}

/// Get hero static data.
#[poise::command(slash_command)]
async fn get_hero(ctx: Context<'_>, name: String) -> Result<(), Error> {
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
    let (data, id) = query_stratz::<constanthero::ConstantGL>(
        &ctx.data().dota_token,
        &name,
        query,
        &ctx.data().search_index,
    )
    .await?;

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
        ctx.data().heroes.get(&id).unwrap(),
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

    Ok(())
}

/// Get hero winrate in the last 4 weeks.
#[poise::command(slash_command)]
async fn get_winrate(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let query = r#"query myQuery($id: Short){
        heroStats {
            winWeek(heroIds: [$id]) {
                winCount matchCount
            }
        }
    }"#;
    let (data, id) = query_stratz::<winrate::DataGL>(
        &ctx.data().dota_token,
        &name,
        query,
        &ctx.data().search_index,
    )
    .await?;

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
        &ctx.data().heroes.get(&id).expect(
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
    let dota_token = secret_store
        .get("DOTA_API")
        .context("'DOTA_API' was not found.")?;

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
            commands: vec![
                hello(),
                get_winrate(),
                get_hero(),
                get_synergies_and_counters(),
            ],
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
                    dota_token,
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
