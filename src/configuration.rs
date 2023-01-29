use secrecy::{Secret, ExposeSecret};

pub enum Environment {
    Development,
    Production
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. `development` and `production` are acceptable.",
                other
            ))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        ))
    }
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("RUST_ENV")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse RUST_ENV");

    let environment_filename = format!("{}.yaml", environment.as_str());
    // Init our config reader
    let settings = config::Config::builder()
        // Add configuration values from a file named `configuration.yaml`
        .add_source(config::File::from(configuration_directory.join("base.yaml")))
        .add_source(config::File::from(configuration_directory.join(&environment_filename)))
        .build()?;
    // Try to convert the configuation values it read into our settings type
    settings.try_deserialize::<Settings>()
}
