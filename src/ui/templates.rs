use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    /// Global Tera instance for template rendering
    pub static ref TEMPLATES: Tera = {
        match Tera::new("templates/ui/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error: {}", e);
                std::process::exit(1);
            }
        }
    };
}
