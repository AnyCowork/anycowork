use crate::schema::settings;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub id: String,
    pub key: String,
    pub value: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = settings)]
pub struct NewSetting {
    pub id: String,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = settings)]
pub struct UpdateSetting {
    pub value: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
}

pub fn get_setting(pool: &crate::database::DbPool, key: &str) -> Option<String> {
    use crate::schema::settings;

    if let Ok(mut conn) = pool.get() {
        settings::table
            .filter(settings::key.eq(key))
            .select(settings::value)
            .first::<Option<String>>(&mut conn)
            .ok()
            .flatten()
    } else {
        None
    }
}
