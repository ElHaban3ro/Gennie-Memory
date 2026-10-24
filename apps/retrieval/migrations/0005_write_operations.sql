CREATE TABLE memory_write_operations (
    operation_id TEXT PRIMARY KEY,
    request_sha256 TEXT NOT NULL CHECK (length(request_sha256) = 64),
    memory_id TEXT NOT NULL REFERENCES memories(id),
    revision INTEGER NOT NULL CHECK (revision > 0),
    committed_at TEXT NOT NULL,
    FOREIGN KEY (memory_id, revision)
        REFERENCES memory_revisions(memory_id, revision)
);
