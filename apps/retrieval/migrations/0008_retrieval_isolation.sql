ALTER TABLE memories ADD COLUMN actor_id TEXT NOT NULL DEFAULT 'local'
    CHECK (length(trim(actor_id)) BETWEEN 1 AND 128);
CREATE INDEX memory_actor_scope_idx ON memories(actor_id, scope_kind, scope_id, entity_id);

-- Derived keys predating actor/entity binding are refreshed lazily on write.
ALTER TABLE memory_content_keys ADD COLUMN identity_version INTEGER NOT NULL DEFAULT 0;
CREATE INDEX memory_content_key_identity ON memory_content_keys(memory_id, revision);

CREATE TABLE entities (
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('person','place','event','company','subject')),
    label TEXT NOT NULL,
    PRIMARY KEY (actor_id, scope_kind, scope_id, entity_id)
);
CREATE TABLE entity_aliases (
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    normalized_label TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (actor_id, scope_kind, scope_id, entity_id, normalized_label),
    FOREIGN KEY (actor_id, scope_kind, scope_id, entity_id)
        REFERENCES entities(actor_id, scope_kind, scope_id, entity_id)
);
CREATE INDEX entity_alias_lookup ON entity_aliases(actor_id, scope_kind, scope_id, normalized_label);
CREATE TABLE entity_operations (
    operation_id TEXT PRIMARY KEY,
    request_sha256 TEXT NOT NULL,
    response_json TEXT NOT NULL,
    committed_at TEXT NOT NULL
);
CREATE TABLE entity_evidence (
    operation_id TEXT PRIMARY KEY REFERENCES entity_operations(operation_id),
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    source_json TEXT NOT NULL,
    FOREIGN KEY (actor_id, scope_kind, scope_id, entity_id)
        REFERENCES entities(actor_id, scope_kind, scope_id, entity_id)
);
INSERT INTO entities(actor_id,scope_kind,scope_id,entity_id,kind,label)
SELECT DISTINCT actor_id,scope_kind,scope_id,entity_id,'subject',entity_id
FROM memories WHERE entity_id IS NOT NULL;
INSERT INTO entity_aliases(actor_id,scope_kind,scope_id,entity_id,normalized_label,label)
SELECT actor_id,scope_kind,scope_id,entity_id,lower(entity_id),entity_id FROM entities;

-- Keep the external-content FTS projection consistent on direct updates too.
CREATE TRIGGER search_documents_au AFTER UPDATE ON search_documents BEGIN
    INSERT INTO memory_fts(memory_fts,rowid,original_text,summary)
    VALUES ('delete',old.doc_id,old.original_text,old.summary);
    INSERT INTO memory_fts(rowid,original_text,summary)
    VALUES (new.doc_id,new.original_text,new.summary);
END;
