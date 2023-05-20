use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub database_url: String,
    pub test_database_url: String,
    pub jwt_secret: String,
    pub ldap_uri: String,
    pub ldap_filter: String,
    pub ldap_uid_column: String,
    pub ldap_user_dn: String,
    pub client_host: Option<String>
}

pub fn get_config() -> envy::Result<Config> {
    dotenv().ok();
    envy::from_env::<Config>()
}
