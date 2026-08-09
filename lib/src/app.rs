pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn name(&self) -> &'static str {
        env!("CARGO_PKG_NAME")
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_is_constructible() {
        let app = App::new();
        assert_eq!(app.name(), "context-manager");
    }

    #[test]
    fn version_is_semver() {
        let app = App::new();
        let parts: Vec<&str> = app.version().split('.').collect();
        assert_eq!(parts.len(), 3);
    }
}
