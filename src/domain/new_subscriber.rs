use serde::{Serialize, Deserialize};

use super::{subscriber_name::SubscriberName, SubscriberEmail};

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSubscriptionForm {
    pub email: String,
    pub name: String
}
