CREATE TABLE pages (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  type TEXT NOT NULL DEFAULT 'page',
  parent_id TEXT,
  day_date TEXT,
  icon TEXT,
  cover_image TEXT,
  is_archived INTEGER NOT NULL DEFAULT 0,
  is_published INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (parent_id) REFERENCES pages(id) ON DELETE CASCADE
);

CREATE INDEX idx_pages_parent ON pages(parent_id);
CREATE INDEX idx_pages_archived ON pages(is_archived);
CREATE INDEX idx_pages_type ON pages(type);

CREATE TABLE blocks (
  id TEXT NOT NULL PRIMARY KEY,
  page_id TEXT NOT NULL,
  type TEXT NOT NULL,
  content_json TEXT NOT NULL,
  order_index INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
);

CREATE INDEX idx_blocks_page ON blocks(page_id);
CREATE INDEX idx_blocks_order ON blocks(page_id, order_index);

CREATE TABLE attachments (
  id TEXT NOT NULL PRIMARY KEY,
  page_id TEXT NOT NULL,
  block_id TEXT,
  file_path TEXT NOT NULL,
  file_name TEXT NOT NULL,
  file_type TEXT NOT NULL,
  file_size INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_page ON attachments(page_id);
