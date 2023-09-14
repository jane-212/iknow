pub mod csgo;
pub mod utils;

#[macro_use]
extern crate log;

lazy_static::lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        match tera::Tera::new("template/**/*") {
            Ok(t) => t,
            Err(e) => {
                error!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        }
    };
}
