#[macro_use]
extern crate rabbithole_derive as rbh_derive;

#[derive(rbh_derive::ModelDecorator)]
#[model(type = "dogs")]
pub struct Dog {
    #[model(id)]
    pub id: String,
    pub name: String,
    #[model(to_many)]
    pub fleas: Vec<Flea>,
    pub master: Master,
}

#[derive(rbh_derive::ModelDecorator)]
#[model(type = "fleas")]
pub struct Flea {
    #[model(id)]
    pub id: String,
    pub name: String,
}

#[derive(rbh_derive::ModelDecorator)]
#[model(type = "masters")]
pub struct Master {
    #[model(id)]
    pub passport_number: String,
    pub name: String,
    #[model(to_one)]
    pub only_flea: Option<Flea>,
    pub gender: Gender,
}

pub enum Gender {
    Male,
    Female,
    Unknown,
}
