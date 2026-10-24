CREATE TABLE memory_write_operations_v7 (
    operation_id TEXT PRIMARY KEY,
    request_sha256 TEXT NOT NULL CHECK (length(request_sha256) = 64),
    memory_id TEXT NOT NULL REFERENCES memories(id),
    revision INTEGER NOT NULL CHECK (revision > 0),
    committed_at TEXT NOT NULL,
    outcome TEXT NOT NULL DEFAULT 'created'
        CHECK (outcome IN ('created', 'deduplicated', 'revised')),
    evidence_added INTEGER NOT NULL DEFAULT 1
        CHECK (evidence_added IN (0, 1)),
    FOREIGN KEY (memory_id, revision)
        REFERENCES memory_revisions(memory_id, revision)
);

INSERT INTO memory_write_operations_v7
    (operation_id, request_sha256, memory_id, revision, committed_at, outcome, evidence_added)
SELECT operation_id, request_sha256, memory_id, revision, committed_at, outcome, evidence_added
FROM memory_write_operations;

DROP TABLE memory_write_operations;
ALTER TABLE memory_write_operations_v7 RENAME TO memory_write_operations;
