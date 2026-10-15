ALTER TABLE memories ADD COLUMN entity_id TEXT;

CREATE INDEX memory_scope_entity_idx
    ON memories(scope_kind, scope_id, entity_id, current_revision);
