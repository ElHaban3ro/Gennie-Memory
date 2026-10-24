ALTER TABLE memory_write_operations
    ADD COLUMN outcome TEXT NOT NULL DEFAULT 'created'
        CHECK (outcome IN ('created', 'deduplicated'));

ALTER TABLE memory_write_operations
    ADD COLUMN evidence_added INTEGER NOT NULL DEFAULT 1
        CHECK (evidence_added IN (0, 1));

CREATE TABLE memory_content_keys (
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64),
    memory_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    PRIMARY KEY (scope_kind, scope_id, content_sha256),
    FOREIGN KEY (memory_id, revision)
        REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);

CREATE TABLE memory_source_evidence (
    evidence_id INTEGER PRIMARY KEY,
    memory_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    source_kind TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    source_locator TEXT,
    evidence_text TEXT NOT NULL,
    verification_status TEXT NOT NULL DEFAULT 'declared_unverified'
        CHECK (verification_status IN ('declared_unverified', 'verified', 'rejected')),
    recorded_at TEXT NOT NULL,
    UNIQUE (memory_id, revision, source_kind, source_ref, source_locator, evidence_text),
    FOREIGN KEY (memory_id, revision)
        REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);

CREATE INDEX memory_source_evidence_memory_idx
    ON memory_source_evidence(memory_id, revision);
