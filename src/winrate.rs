use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct DataGL {
    pub heroStats: HeroStats,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct HeroStats {
    pub winWeek: Vec<HeroWinCount>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct HeroWinCount {
    pub winCount: f64,
    pub matchCount: f64,
}
