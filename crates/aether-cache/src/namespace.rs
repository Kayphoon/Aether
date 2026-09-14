#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKeyNamespace {
    prefix: String,
}

impl CacheKeyNamespace {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    pub fn child(&self, suffix: &str) -> Self {
        if self.prefix.is_empty() {
            return Self::new(suffix);
        }
        if suffix.is_empty() {
            return self.clone();
        }
        Self::new(format!("{}:{}", self.prefix, suffix))
    }

    pub fn key(&self, raw_key: &str) -> String {
        if self.prefix.is_empty() {
            return raw_key.to_string();
        }
        if raw_key.is_empty() {
            return self.prefix.clone();
        }
        format!("{}:{}", self.prefix, raw_key)
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

#[cfg(test)]
mod tests {
    use super::CacheKeyNamespace;

    #[test]
    fn composes_scoped_keys() {
        let root = CacheKeyNamespace::new("aether");
        let child = root.child("auth");

        assert_eq!(root.key("user-1"), "aether:user-1");
        assert_eq!(child.key("user-1"), "aether:auth:user-1");
        assert_eq!(child.prefix(), "aether:auth");
    }
}
