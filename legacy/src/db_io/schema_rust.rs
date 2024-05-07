use super::schema::v1::messages;
use diesel::prelude::Insertable;
use serde_derive::Serialize;

#[derive(Serialize, Insertable)]
#[serde(deny_unknown_fields)]
#[diesel(table_name = messages)]
pub struct Message {
    pub title: String,
    pub body: String,
    pub name: String,
}