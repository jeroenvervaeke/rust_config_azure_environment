use config::{ConfigError, Environment, Source, Value};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct AzureEnvironment {
    environment: Environment,
}

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref KEY_REGEX: Regex = Regex::new(r"\.(\d+)($|\.)").unwrap();
}

impl AzureEnvironment {
    pub fn from_environment(environment: Environment) -> Self {
        Self { environment }
    }

    pub fn with_prefix(s: &str) -> Self {
        Environment::with_prefix(s).into()
    }

    pub fn prefix(self, s: &str) -> Self {
        self.environment.prefix(s).into()
    }

    pub fn separator(self, s: &str) -> Self {
        self.environment.separator(s).into()
    }

    pub fn ignore_empty(self, ignore: bool) -> Self {
        self.environment.ignore_empty(ignore).into()
    }

    // Changes keys like this ".0" into "[0]"
    fn create_array_key(key: &str) -> String {
        let out = KEY_REGEX.replace_all(&key, r"[$1]$2").into();
        out
    }
}

impl From<Environment> for AzureEnvironment {
    fn from(environment: Environment) -> Self {
        Self::from_environment(environment)
    }
}

impl Source for AzureEnvironment {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        // The underlying Environment turns replaces the separator with a .
        // create_array_key replaces patterns like ".0" into "[0]"
        Ok(self
            .environment
            .collect()?
            .into_iter()
            .map(|(key, value)| (Self::create_array_key(&key), value))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;

    #[test]
    fn test_key_replace_last() {
        let input = "EXTERNAL_APIS.AUTH.SCOPES.0";
        let expected = "EXTERNAL_APIS.AUTH.SCOPES[0]";

        let actual = AzureEnvironment::create_array_key(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_replace_in_between() {
        let input = "EXTERNAL_APIS.AUTH.CLIENT.1.SECRET";
        let expected = "EXTERNAL_APIS.AUTH.CLIENT[1].SECRET";

        let actual = AzureEnvironment::create_array_key(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_replace_multiple() {
        let input = "EXTERNAL_APIS.AUTH.CLIENT.10.SCOPES.999";
        let expected = "EXTERNAL_APIS.AUTH.CLIENT[10].SCOPES[999]";

        let actual = AzureEnvironment::create_array_key(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_replace_digit_prefixed() {
        let input = "WEIRD_KEY.0A";
        let expected = "WEIRD_KEY.0A";

        let actual = AzureEnvironment::create_array_key(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn deserialize_array_setting() {
        use serde::Deserialize;
        use std::env;

        #[derive(Debug, Deserialize, PartialEq)]
        struct TestSettings {
            value_1: u64,
            other_values: Vec<String>,
        }

        env::set_var("TEST_VALUE_1", "42");
        env::set_var("TEST_OTHER_VALUES__0", "value_1");
        env::set_var("TEST_OTHER_VALUES__1", "value_2");

        let expected = TestSettings {
            value_1: 42,
            other_values: vec!["value_1".to_string(), "value_2".to_string()],
        };

        let actual: TestSettings = {
            let mut s = Config::new();
            s.merge(AzureEnvironment::with_prefix("TEST").separator("__")).unwrap();
            s.try_into().unwrap()
        };

        assert_eq!(expected, actual);
    }
}
