ALTER TABLE memories ADD COLUMN topic_key TEXT;
CREATE UNIQUE INDEX memory_topic_identity ON memories(actor_id,scope_kind,scope_id,topic_key)
    WHERE topic_key IS NOT NULL;

CREATE TABLE reconciliation_operations (
    operation_id TEXT PRIMARY KEY,
    request_sha256 TEXT NOT NULL CHECK(length(request_sha256)=64),
    response_json TEXT NOT NULL,
    committed_at TEXT NOT NULL
);
CREATE TABLE memory_proposals (
    proposal_id TEXT PRIMARY KEY,
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    request_json TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE memory_proposal_targets (
    proposal_id TEXT NOT NULL REFERENCES memory_proposals(proposal_id) ON DELETE CASCADE,
    memory_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    relation TEXT NOT NULL,
    reason TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    PRIMARY KEY(proposal_id,memory_id),
    FOREIGN KEY(memory_id,revision) REFERENCES memory_revisions(memory_id,revision) ON DELETE CASCADE
);
CREATE TABLE memory_relations (
    from_memory_id TEXT NOT NULL,
    from_revision INTEGER NOT NULL,
    to_memory_id TEXT NOT NULL,
    to_revision INTEGER NOT NULL,
    relation TEXT NOT NULL CHECK(relation IN ('duplicate','complement','correction','exception','contradiction','unrelated')),
    operation_id TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    PRIMARY KEY(from_memory_id,from_revision,to_memory_id,to_revision,relation),
    FOREIGN KEY(from_memory_id,from_revision) REFERENCES memory_revisions(memory_id,revision) ON DELETE CASCADE,
    FOREIGN KEY(to_memory_id,to_revision) REFERENCES memory_revisions(memory_id,revision) ON DELETE CASCADE
);
CREATE TABLE memory_evidence_audit (
    audit_id INTEGER PRIMARY KEY,
    operation_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    memory_id TEXT,
    revision INTEGER,
    proposal_id TEXT,
    action TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    evidence_sha256 TEXT NOT NULL CHECK(length(evidence_sha256)=64),
    provenance_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL
);
CREATE INDEX memory_evidence_audit_memory ON memory_evidence_audit(memory_id,revision,audit_id);

-- Global operation IDs must not be reused by different public write surfaces.
CREATE TRIGGER reconciliation_operation_collision BEFORE INSERT ON reconciliation_operations
WHEN EXISTS(SELECT 1 FROM entity_operations WHERE operation_id=NEW.operation_id)
BEGIN SELECT RAISE(ABORT,'idempotency_conflict'); END;
CREATE TRIGGER entity_reconciliation_collision BEFORE INSERT ON entity_operations
WHEN EXISTS(SELECT 1 FROM reconciliation_operations WHERE operation_id=NEW.operation_id)
BEGIN SELECT RAISE(ABORT,'idempotency_conflict'); END;
