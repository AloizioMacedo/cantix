use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct HeroQuery {
    pub heroStats: MatchUp,
}

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct MatchUp {
    pub matchUp: Vec<Hero>,
}

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct Hero {
    pub with: Vec<OtherHero>,
    pub vs: Vec<OtherHero>,
}

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct OtherHero {
    pub heroId2: u8,
    pub winsAverage: f32,
}
