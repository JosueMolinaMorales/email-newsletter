use serde::{Serialize, Deserialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscriber {
    pub email: String,
    pub name: SubscriberName,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSubscriptionForm {
    pub email: String,
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriberName(String);

impl SubscriberName {

    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.
    /// It panics otherwise
    pub fn parse(name: String) -> SubscriberName {
        let is_empty_or_whitespace = name.trim().is_empty();
        let is_too_long = name.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = name.chars().any(|g| forbidden_characters.contains(&g));
    
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name", name)
        }

        Self(name)
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}