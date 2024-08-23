pub trait Key: Send+Sync {
    fn check(&self, token: &str) -> Result<(), String>;

    fn should_show_input(&self) -> bool {
        true
    }
}

impl Key for () {
    fn check(&self, token: &str) -> Result<(), String> {
        let _ = token;
        Ok(())
    }

    fn should_show_input(&self) -> bool {
        false
    }
}

impl Key for str {
    fn check(&self, token: &str) -> Result<(), String> {
        if token == self {
            Ok(())
        } else {
            Err(String::new())
        }
    }
}

impl Key for String {
    fn check(&self, token: &str) -> Result<(), String> {
        self[..].check(token)
    }
}

impl<T> Key for T where T: Fn(&str) -> Result<(), String> + Send + Sync {
    fn check(&self, token: &str) -> Result<(), String> {
        self(token)
    }
}
