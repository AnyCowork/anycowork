use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::pages)]
pub struct Page {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub parent_id: Option<String>,
    pub day_date: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub is_archived: i32,
    pub is_published: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::pages)]
pub struct NewPage {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub parent_id: Option<String>,
    pub day_date: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub is_archived: i32,
    pub is_published: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::pages)]
pub struct UpdatePage {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub icon: Option<String>,
    pub cover_image: Option<String>,
    pub is_archived: Option<i32>,
    pub is_published: Option<i32>,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::blocks)]
pub struct Block {
    pub id: String,
    pub page_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub content_json: String,
    pub order_index: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::blocks)]
pub struct NewBlock {
    pub id: String,
    pub page_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub content_json: String,
    pub order_index: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::blocks)]
pub struct UpdateBlock {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub content_json: Option<String>,
    pub order_index: Option<i32>,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::attachments)]
pub struct Attachment {
    pub id: String,
    pub page_id: String,
    pub block_id: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::attachments)]
pub struct NewAttachment {
    pub id: String,
    pub page_id: String,
    pub block_id: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i32,
    pub created_at: chrono::NaiveDateTime,
}
