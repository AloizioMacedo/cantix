use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ConstantGL {
    pub constants: ConstantQuery,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ConstantQuery {
    pub hero: HeroConstantQuery,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct HeroConstantQuery {
    pub stats: HeroData,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct HeroData {
    pub attackType: String,
    pub startingArmor: f32,
    pub startingDamageMin: f32,
    pub startingDamageMax: f32,
    pub attackRate: f32,
    pub attackRange: f32,
    pub primaryAttribute: String,
    pub strengthBase: u16,
    pub strengthGain: f32,
    pub intelligenceBase: u16,
    pub intelligenceGain: f32,
    pub agilityBase: u16,
    pub agilityGain: f32,
    pub hpRegen: f32,
    pub mpRegen: f32,
    pub moveSpeed: f32,
    pub moveTurnRate: f32,
}
