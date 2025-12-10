use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    /// Global Tera instance for template rendering
    pub static ref TEMPLATES: Tera = {
        match Tera::new("templates/ui/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error: {}", e);
                eprintln!("Please ensure template files exist in the 'templates/ui/' directory");
                eprintln!("Expected files: layout.html, queue.html, experiment.html");
                std::process::exit(1);
            }
        }
    };
}
