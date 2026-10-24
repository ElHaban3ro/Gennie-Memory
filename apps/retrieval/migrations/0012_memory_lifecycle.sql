ALTER TABLE memories ADD COLUMN lifecycle_state TEXT NOT NULL DEFAULT 'active' CHECK(lifecycle_state IN ('active','withdrawn'));
CREATE TABLE memory_review_schedule (
    memory_id TEXT PRIMARY KEY REFERENCES memories(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL,
    next_review_at TEXT,
    reviewed_at TEXT
);
INSERT INTO memory_review_schedule(memory_id,revision) SELECT id,current_revision FROM memories;
CREATE TABLE memory_lifecycle_operations (
    operation_id TEXT PRIMARY KEY,
    request_sha256 TEXT NOT NULL,
    response_json TEXT NOT NULL,
    committed_at TEXT NOT NULL
);
-- A target opts a scope/profile into indexing. No automatic model loading.
CREATE TABLE embedding_jobs (
    job_id TEXT PRIMARY KEY,
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    space_id TEXT NOT NULL,
    generation INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL CHECK(status IN ('pending','running','completed','failed','cancelled')),
    lease_token TEXT,
    lease_until TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    index_id TEXT,
    updated_at TEXT NOT NULL,
    UNIQUE(actor_id,scope_kind,scope_id,space_id)
);
CREATE TRIGGER memory_review_create AFTER INSERT ON memories BEGIN
    INSERT INTO memory_review_schedule(memory_id,revision) VALUES(NEW.id,NEW.current_revision);
    UPDATE embedding_jobs SET generation=generation+1,status='pending',lease_token=NULL,lease_until=NULL,error_code=NULL,index_id=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE actor_id=NEW.actor_id AND scope_kind=NEW.scope_kind AND scope_id=NEW.scope_id;
END;
CREATE TRIGGER memory_lifecycle_no_implicit_restore BEFORE UPDATE OF current_revision ON memories
WHEN OLD.lifecycle_state!='active' AND NEW.lifecycle_state!='active' BEGIN SELECT RAISE(ABORT,'memory_withdrawn'); END;
CREATE TRIGGER memory_review_revision AFTER UPDATE OF current_revision ON memories BEGIN
    UPDATE memory_review_schedule SET revision=NEW.current_revision,next_review_at=NULL,reviewed_at=NULL WHERE memory_id=NEW.id;
END;
CREATE TRIGGER memory_embedding_changed AFTER UPDATE OF current_revision,lifecycle_state ON memories BEGIN
    UPDATE embedding_jobs SET generation=generation+1,status='pending',lease_token=NULL,lease_until=NULL,error_code=NULL,index_id=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE actor_id=NEW.actor_id AND scope_kind=NEW.scope_kind AND scope_id=NEW.scope_id;
END;
CREATE TRIGGER memory_embedding_deleted AFTER DELETE ON memories BEGIN
    UPDATE embedding_jobs SET generation=generation+1,status='pending',lease_token=NULL,lease_until=NULL,error_code=NULL,index_id=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE actor_id=OLD.actor_id AND scope_kind=OLD.scope_kind AND scope_id=OLD.scope_id;
END;

-- Minimal tombstones prevent replay of erased legacy writes from recreating content.
CREATE TABLE memory_erased_write_operations (
    operation_id TEXT PRIMARY KEY,
    request_sha256 TEXT NOT NULL,
    erased_at TEXT NOT NULL
);
CREATE TRIGGER lifecycle_write_collision BEFORE INSERT ON memory_write_operations
WHEN EXISTS(SELECT 1 FROM memory_lifecycle_operations WHERE operation_id=NEW.operation_id UNION ALL SELECT 1 FROM memory_erased_write_operations WHERE operation_id=NEW.operation_id)
BEGIN SELECT RAISE(ABORT,'idempotency_conflict_or_content_deleted'); END;
CREATE TRIGGER lifecycle_entity_collision BEFORE INSERT ON entity_operations
WHEN EXISTS(SELECT 1 FROM memory_lifecycle_operations WHERE operation_id=NEW.operation_id UNION ALL SELECT 1 FROM memory_erased_write_operations WHERE operation_id=NEW.operation_id)
BEGIN SELECT RAISE(ABORT,'idempotency_conflict_or_content_deleted'); END;
CREATE TRIGGER lifecycle_reconciliation_collision BEFORE INSERT ON reconciliation_operations
WHEN EXISTS(SELECT 1 FROM memory_lifecycle_operations WHERE operation_id=NEW.operation_id UNION ALL SELECT 1 FROM memory_erased_write_operations WHERE operation_id=NEW.operation_id)
BEGIN SELECT RAISE(ABORT,'idempotency_conflict_or_content_deleted'); END;
CREATE TRIGGER erased_lifecycle_collision BEFORE INSERT ON memory_lifecycle_operations
WHEN EXISTS(SELECT 1 FROM memory_erased_write_operations WHERE operation_id=NEW.operation_id)
BEGIN SELECT RAISE(ABORT,'idempotency_conflict_or_content_deleted'); END;
