CREATE TABLE source_snapshots (
    sequence INTEGER PRIMARY KEY,
    snapshot_id TEXT NOT NULL UNIQUE CHECK (length(snapshot_id) = 64),
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    version TEXT NOT NULL,
    source_text TEXT NOT NULL,
    text_sha256 TEXT NOT NULL CHECK (length(text_sha256) = 64),
    registered_at TEXT NOT NULL,
    UNIQUE (actor_id, scope_kind, scope_id, source_kind, source_ref, version)
);
CREATE INDEX source_snapshots_current_idx ON source_snapshots
    (actor_id, scope_kind, scope_id, source_kind, source_ref, sequence DESC);

CREATE TABLE source_verification_checks (
    check_id TEXT PRIMARY KEY CHECK (length(check_id) = 64),
    memory_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    snapshot_id TEXT REFERENCES source_snapshots(snapshot_id),
    status TEXT NOT NULL CHECK (status IN ('verified_literal', 'rejected', 'declared_unverified', 'not_provided')),
    reason_code TEXT NOT NULL,
    report_json TEXT NOT NULL,
    checked_at TEXT NOT NULL,
    FOREIGN KEY (memory_id, revision) REFERENCES memory_revisions(memory_id, revision)
);
CREATE INDEX source_verification_memory_idx ON source_verification_checks(memory_id, revision);
