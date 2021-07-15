use rocket::{Phase, Rocket};

/// Config values for the API service. Available as a rocket request
/// guard.
pub struct Config {
    /// The list of origins allowed to access the service.
    pub allowed_origins: Vec<String>,

    /// The secret key for signing JWTs.
    pub jwt_secret: String,
}

pub fn create_config<T: Phase>(rocket: &Rocket<T>) -> Config {
    let figment = rocket.figment();
    let allowed_origins: Vec<String> = figment.extract_inner("origins").expect("origins");
    let jwt_secret: String = figment.extract_inner("jwt_secret").expect("jwt_secret");

    Config {
        allowed_origins,
        jwt_secret,
    }
}
