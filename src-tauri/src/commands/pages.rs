use crate::models::{
    Attachment, Block, NewAttachment, NewBlock, NewPage, Page, UpdateBlock, UpdatePage,
};
use crate::schema;
use crate::AppState;
use diesel::prelude::*;
use serde::Deserialize;
use tauri::State;

// ============================================================================
// PAGE COMMANDS
// ============================================================================

#[tauri::command]
pub async fn create_page(
    state: State<'_, AppState>,
    title: String,
    page_type: String,
    parent_id: Option<String>,
) -> Result<Page, String> {
    use schema::pages;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let new_page = NewPage {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        type_: page_type,
        parent_id,
        day_date: None,
        icon: None,
        cover_image: None,
        is_archived: 0,
        is_published: 0,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(pages::table)
        .values(&new_page)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_page.id.clone();

    pages::table
        .filter(pages::id.eq(created_id))
        .first::<Page>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pages(
    state: State<'_, AppState>,
    parent_id_param: Option<String>,
    archived: Option<bool>,
) -> Result<Vec<Page>, String> {
    use schema::pages::dsl::{is_archived, pages, parent_id, updated_at};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let mut query = pages.into_boxed();

    if let Some(pid) = parent_id_param {
        query = query.filter(parent_id.eq(pid));
    } else {
        query = query.filter(parent_id.is_null());
    }

    if let Some(arch) = archived {
        let arch_val = if arch { 1 } else { 0 };
        query = query.filter(is_archived.eq(arch_val));
    }

    query
        .order(updated_at.desc())
        .load::<Page>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_page(state: State<'_, AppState>, page_id: String) -> Result<Page, String> {
    use schema::pages::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    pages
        .filter(id.eq(page_id))
        .first::<Page>(&mut conn)
        .map_err(|e| format!("Page not found: {}", e))
}

#[tauri::command]
pub async fn update_page(
    state: State<'_, AppState>,
    page_id: String,
    title_param: Option<String>,
    icon_param: Option<String>,
    cover_image_param: Option<String>,
    is_published_param: Option<bool>,
) -> Result<Page, String> {
    use schema::pages::dsl::{id, pages};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let update = UpdatePage {
        title: title_param,
        type_: None,
        icon: icon_param,
        cover_image: cover_image_param,
        is_archived: None,
        is_published: is_published_param.map(|v| if v { 1 } else { 0 }),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(pages.filter(id.eq(&page_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    pages
        .filter(id.eq(&page_id))
        .first::<Page>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_page(state: State<'_, AppState>, page_id: String) -> Result<Page, String> {
    use schema::pages::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let update = UpdatePage {
        title: None,
        type_: None,
        icon: None,
        cover_image: None,
        is_archived: Some(1),
        is_published: None,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(pages.filter(id.eq(&page_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    pages
        .filter(id.eq(&page_id))
        .first::<Page>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_page(state: State<'_, AppState>, page_id: String) -> Result<Page, String> {
    use schema::pages::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let update = UpdatePage {
        title: None,
        type_: None,
        icon: None,
        cover_image: None,
        is_archived: Some(0),
        is_published: None,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(pages.filter(id.eq(&page_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    pages
        .filter(id.eq(&page_id))
        .first::<Page>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_page(state: State<'_, AppState>, page_id: String) -> Result<(), String> {
    use schema::pages::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    diesel::delete(pages.filter(id.eq(page_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// BLOCK COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_page_blocks(
    state: State<'_, AppState>,
    page_id_param: String,
) -> Result<Vec<Block>, String> {
    use schema::blocks::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    blocks
        .filter(page_id.eq(page_id_param))
        .order(order_index.asc())
        .load::<Block>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_block(
    state: State<'_, AppState>,
    page_id_param: String,
    block_type: String,
    content_json_param: String,
    order_index_param: Option<i32>,
) -> Result<Block, String> {
    use schema::blocks;
    use schema::blocks::dsl::{blocks as blocks_table, id as block_id, order_index, page_id};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // If order_index not provided, find max and add 1
    let computed_order = if let Some(idx) = order_index_param {
        idx
    } else {
        let max_order: Option<i32> = blocks_table
            .filter(page_id.eq(&page_id_param))
            .select(diesel::dsl::max(order_index))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;
        max_order.unwrap_or(0) + 1
    };

    let new_block = NewBlock {
        id: uuid::Uuid::new_v4().to_string(),
        page_id: page_id_param,
        type_: block_type,
        content_json: content_json_param,
        order_index: computed_order,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(blocks::table)
        .values(&new_block)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_block.id.clone();

    blocks_table
        .filter(block_id.eq(created_id))
        .first::<Block>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_block(
    state: State<'_, AppState>,
    block_id: String,
    block_type: Option<String>,
    content_json_param: Option<String>,
    order_index_param: Option<i32>,
) -> Result<Block, String> {
    use schema::blocks::dsl::{blocks, id};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let update = UpdateBlock {
        type_: block_type,
        content_json: content_json_param,
        order_index: order_index_param,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(blocks.filter(id.eq(&block_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    blocks
        .filter(id.eq(&block_id))
        .first::<Block>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_block(state: State<'_, AppState>, block_id: String) -> Result<(), String> {
    use schema::blocks::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    diesel::delete(blocks.filter(id.eq(block_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Deserialize)]
pub struct BlockUpdate {
    pub id: String,
    pub block_type: Option<String>,
    pub content_json: Option<String>,
    pub order_index: Option<i32>,
}

#[tauri::command]
pub async fn batch_update_blocks(
    state: State<'_, AppState>,
    page_id_param: String,
    updates: Vec<BlockUpdate>,
) -> Result<Vec<Block>, String> {
    use schema::blocks::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Update each block
    for update_data in updates {
        let update = UpdateBlock {
            type_: update_data.block_type,
            content_json: update_data.content_json,
            order_index: update_data.order_index,
            updated_at: chrono::Utc::now().naive_utc(),
        };

        diesel::update(blocks.filter(id.eq(&update_data.id)))
            .set(&update)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
    }

    // Return all blocks for the page
    blocks
        .filter(page_id.eq(page_id_param))
        .order(order_index.asc())
        .load::<Block>(&mut conn)
        .map_err(|e| e.to_string())
}

// ============================================================================
// ATTACHMENT COMMANDS
// ============================================================================

#[tauri::command]
pub async fn upload_attachment(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    page_id_param: String,
    file_name: String,
    file_data: Vec<u8>,
) -> Result<Attachment, String> {
    use schema::attachments;
    use std::fs;
    use tauri::Manager;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let attachments_base = app_data_dir.join("attachments");
    let attachments_dir = attachments_base.join(&page_id_param);
    fs::create_dir_all(&attachments_dir).map_err(|e| e.to_string())?;

    // Save file
    let file_path = attachments_dir.join(&file_name);
    fs::write(&file_path, &file_data).map_err(|e| e.to_string())?;

    // Determine file type from extension
    let file_type = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let new_attachment = NewAttachment {
        id: uuid::Uuid::new_v4().to_string(),
        page_id: page_id_param,
        block_id: None,
        file_path: file_path.to_string_lossy().to_string(),
        file_name,
        file_type,
        file_size: file_data.len() as i32,
        created_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(attachments::table)
        .values(&new_attachment)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_attachment.id.clone();

    attachments::table
        .filter(attachments::id.eq(created_id))
        .first::<Attachment>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_page_attachments(
    state: State<'_, AppState>,
    page_id_param: String,
) -> Result<Vec<Attachment>, String> {
    use schema::attachments::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    attachments
        .filter(page_id.eq(page_id_param))
        .order(created_at.desc())
        .load::<Attachment>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_attachment(
    _app: tauri::AppHandle, // Included for consistency/future use if needed, though file_path is already absolute
    state: State<'_, AppState>,
    attachment_id: String,
) -> Result<(), String> {
    use schema::attachments::dsl::*;
    use std::fs;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get attachment to delete file
    let attachment: Attachment = attachments
        .filter(id.eq(&attachment_id))
        .first(&mut conn)
        .map_err(|e| e.to_string())?;

    // Delete file from filesystem
    let _ = fs::remove_file(&attachment.file_path);

    // Delete from database
    diesel::delete(attachments.filter(id.eq(attachment_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_test_pool;
    use tauri::test::mock_builder;
    use tauri::Manager;

    fn create_app_state() -> crate::AppState {
        let pool = create_test_pool();
        crate::AppState {
            db_pool: pool,
            pending_approvals: std::sync::Arc::new(dashmap::DashMap::new()),
            telegram_manager: std::sync::Arc::new(crate::telegram::TelegramBotManager::new(
                create_test_pool(),
            )),
            permission_manager: std::sync::Arc::new(crate::permissions::PermissionManager::new()),
        }
    }

    #[tokio::test]
    async fn test_create_and_get_pages() {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let state = create_app_state();
        app.manage(state);
        let state_handle = app.state::<crate::AppState>();

        // Test create_page
        let page = create_page(
            state_handle.clone().into(),
            "Test Page".to_string(),
            "page".to_string(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(page.title, "Test Page");

        // Test get_pages
        let pages = get_pages(state_handle.clone().into(), None, Some(false))
            .await
            .unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].id, page.id);
    }

    #[tokio::test]
    async fn test_create_and_get_blocks() {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let state = create_app_state();
        app.manage(state);
        let state_handle = app.state::<crate::AppState>();

        let page = create_page(
            state_handle.clone().into(),
            "Test Page".to_string(),
            "page".to_string(),
            None,
        )
        .await
        .unwrap();

        // Test create_block
        let block = create_block(
            state_handle.clone().into(),
            page.id.clone(),
            "text".to_string(),
            "{\"text\": \"hello\"}".to_string(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(block.type_, "text");

        // Test get_page_blocks
        let blocks = get_page_blocks(state_handle.clone().into(), page.id)
            .await
            .unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, block.id);
    }
}
