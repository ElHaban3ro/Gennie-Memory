-- Esquema propuesto de iROMEM v0.1. No es una migración de una aplicación existente.
PRAGMA foreign_keys = ON;

CREATE TABLE principals (
  principal_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('user', 'agent'))
);
CREATE TABLE scopes (
  kind TEXT NOT NULL CHECK (kind IN ('user', 'agent', 'project', 'conversation')),
  scope_id TEXT NOT NULL,
  PRIMARY KEY (kind, scope_id)
);
CREATE TABLE scope_members (
  scope_kind TEXT NOT NULL,
  scope_id TEXT NOT NULL,
  principal_id TEXT NOT NULL REFERENCES principals(principal_id),
  can_read INTEGER NOT NULL CHECK (can_read IN (0, 1)),
  can_write INTEGER NOT NULL CHECK (can_write IN (0, 1)),
  PRIMARY KEY (scope_kind, scope_id, principal_id),
  FOREIGN KEY (scope_kind, scope_id) REFERENCES scopes(kind, scope_id)
);
CREATE TABLE memories (
  memory_id TEXT PRIMARY KEY,
  owner_id TEXT NOT NULL REFERENCES principals(principal_id),
  scope_kind TEXT NOT NULL,
  scope_id TEXT NOT NULL,
  topic_key TEXT,
  current_revision INTEGER NOT NULL CHECK (current_revision >= 1),
  current_fingerprint TEXT NOT NULL,
  visibility TEXT NOT NULL CHECK (visibility IN ('private', 'shared')),
  sensitive INTEGER NOT NULL CHECK (sensitive IN (0, 1)),
  withdrawn INTEGER NOT NULL DEFAULT 0 CHECK (withdrawn IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (scope_kind, scope_id) REFERENCES scopes(kind, scope_id),
  FOREIGN KEY (memory_id, current_revision)
    REFERENCES memory_revisions(memory_id, revision) DEFERRABLE INITIALLY DEFERRED,
  UNIQUE (owner_id, scope_kind, scope_id, topic_key),
  UNIQUE (owner_id, scope_kind, scope_id, current_fingerprint)
);
CREATE TABLE memory_grants (
  memory_id TEXT NOT NULL REFERENCES memories(memory_id) ON DELETE CASCADE,
  principal_id TEXT NOT NULL REFERENCES principals(principal_id),
  can_write INTEGER NOT NULL DEFAULT 0 CHECK (can_write IN (0, 1)),
  PRIMARY KEY (memory_id, principal_id)
);
CREATE TABLE memory_revisions (
  memory_id TEXT NOT NULL REFERENCES memories(memory_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK (revision >= 1),
  original_text TEXT NOT NULL CHECK (length(trim(original_text)) BETWEEN 1 AND 16000),
  summary TEXT NOT NULL CHECK (length(trim(summary)) BETWEEN 1 AND 1600),
  type TEXT NOT NULL CHECK (type IN ('preference', 'fact', 'decision', 'event', 'procedure')),
  assertion TEXT NOT NULL CHECK (assertion IN ('direct', 'inferred')),
  source_json TEXT NOT NULL CHECK (json_valid(source_json)),
  conditions TEXT,
  occurred_on TEXT,
  valid_from TEXT,
  valid_until TEXT,
  review_after TEXT,
  importance REAL NOT NULL CHECK (importance BETWEEN 0 AND 1),
  created_at TEXT NOT NULL,
  PRIMARY KEY (memory_id, revision),
  CHECK (valid_from IS NULL OR valid_until IS NULL OR valid_from < valid_until)
);
CREATE TABLE entities (
  entity_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('person', 'place', 'event', 'company', 'subject')),
  label TEXT NOT NULL,
  scope_kind TEXT NOT NULL,
  scope_id TEXT NOT NULL,
  FOREIGN KEY (scope_kind, scope_id) REFERENCES scopes(kind, scope_id)
);
CREATE TABLE revision_entities (
  memory_id TEXT NOT NULL,
  revision INTEGER NOT NULL,
  entity_id TEXT NOT NULL REFERENCES entities(entity_id),
  PRIMARY KEY (memory_id, revision, entity_id),
  FOREIGN KEY (memory_id, revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);
CREATE TABLE memory_evidence (
  evidence_id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL,
  revision INTEGER NOT NULL,
  original_text TEXT NOT NULL,
  source_json TEXT NOT NULL CHECK (json_valid(source_json)),
  confirmed_at TEXT NOT NULL,
  FOREIGN KEY (memory_id, revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);
CREATE TABLE memory_relations (
  relation_id TEXT PRIMARY KEY,
  from_memory_id TEXT NOT NULL,
  from_revision INTEGER NOT NULL,
  to_memory_id TEXT NOT NULL,
  to_revision INTEGER NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('related', 'complements', 'corrects', 'contradicts')),
  reason TEXT NOT NULL,
  FOREIGN KEY (from_memory_id, from_revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE,
  FOREIGN KEY (to_memory_id, to_revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE,
  UNIQUE (from_memory_id, from_revision, to_memory_id, to_revision, kind)
);
CREATE TABLE proposals (
  proposal_id TEXT PRIMARY KEY,
  actor_id TEXT NOT NULL REFERENCES principals(principal_id),
  payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
  reason TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE proposal_targets (
  proposal_id TEXT NOT NULL REFERENCES proposals(proposal_id) ON DELETE CASCADE,
  memory_id TEXT NOT NULL,
  revision INTEGER NOT NULL,
  PRIMARY KEY (proposal_id, memory_id, revision),
  FOREIGN KEY (memory_id, revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);
CREATE TABLE operations (
  actor_id TEXT NOT NULL REFERENCES principals(principal_id),
  operation_id TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('processing', 'committed', 'needs_resolution', 'failed')),
  response_json TEXT CHECK (response_json IS NULL OR json_valid(response_json)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (actor_id, operation_id)
);

CREATE TABLE search_documents (
  doc_id INTEGER PRIMARY KEY,
  memory_id TEXT NOT NULL,
  revision INTEGER NOT NULL,
  original_text TEXT NOT NULL,
  summary TEXT NOT NULL,
  UNIQUE (memory_id, revision),
  FOREIGN KEY (memory_id, revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);
CREATE VIRTUAL TABLE memory_fts USING fts5(
  original_text, summary,
  content='search_documents', content_rowid='doc_id',
  tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER search_documents_ai AFTER INSERT ON search_documents BEGIN
  INSERT INTO memory_fts(rowid, original_text, summary)
  VALUES (new.doc_id, new.original_text, new.summary);
END;
CREATE TRIGGER search_documents_ad AFTER DELETE ON search_documents BEGIN
  INSERT INTO memory_fts(memory_fts, rowid, original_text, summary)
  VALUES ('delete', old.doc_id, old.original_text, old.summary);
END;
CREATE TRIGGER search_documents_au AFTER UPDATE ON search_documents BEGIN
  INSERT INTO memory_fts(memory_fts, rowid, original_text, summary)
  VALUES ('delete', old.doc_id, old.original_text, old.summary);
  INSERT INTO memory_fts(rowid, original_text, summary)
  VALUES (new.doc_id, new.original_text, new.summary);
END;

CREATE TABLE embedding_spaces (
  space_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL,
  model_revision TEXT NOT NULL,
  artifact_hash TEXT NOT NULL,
  tokenizer_hash TEXT NOT NULL,
  runtime_fingerprint TEXT NOT NULL,
  pipeline_version TEXT NOT NULL,
  dimensions INTEGER NOT NULL CHECK (dimensions > 0),
  max_tokens INTEGER NOT NULL CHECK (max_tokens > 0),
  model_path TEXT,
  UNIQUE (space_id, dimensions)
);
CREATE TABLE embedding_jobs (
  memory_id TEXT NOT NULL,
  revision INTEGER NOT NULL,
  space_id TEXT NOT NULL REFERENCES embedding_spaces(space_id),
  status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'ready', 'failed')),
  attempts INTEGER NOT NULL DEFAULT 0,
  expected_chunks INTEGER CHECK (expected_chunks IS NULL OR expected_chunks > 0),
  lease_until TEXT,
  retry_at TEXT,
  last_error_code TEXT,
  PRIMARY KEY (memory_id, revision, space_id),
  FOREIGN KEY (memory_id, revision)
    REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);
CREATE TABLE memory_embeddings (
  memory_id TEXT NOT NULL,
  revision INTEGER NOT NULL,
  space_id TEXT NOT NULL,
  representation TEXT NOT NULL CHECK (representation IN ('original', 'summary')),
  chunk_no INTEGER NOT NULL CHECK (chunk_no >= 0),
  token_start INTEGER NOT NULL CHECK (token_start >= 0),
  token_end INTEGER NOT NULL CHECK (token_end > token_start),
  input_hash TEXT NOT NULL,
  dimensions INTEGER NOT NULL,
  vector BLOB NOT NULL CHECK (length(vector) = dimensions * 4),
  PRIMARY KEY (memory_id, revision, space_id, representation, chunk_no),
  FOREIGN KEY (space_id, dimensions)
    REFERENCES embedding_spaces(space_id, dimensions),
  FOREIGN KEY (memory_id, revision, space_id)
    REFERENCES embedding_jobs(memory_id, revision, space_id) ON DELETE CASCADE
);

CREATE INDEX memory_scope_idx ON memories(scope_kind, scope_id, owner_id, withdrawn);
CREATE INDEX revision_type_time_idx ON memory_revisions(type, valid_until, review_after);
CREATE INDEX entity_memory_idx ON revision_entities(entity_id, memory_id, revision);
CREATE INDEX grant_actor_idx ON memory_grants(principal_id, memory_id);
CREATE INDEX relation_target_idx ON memory_relations(to_memory_id, to_revision);
CREATE INDEX proposal_target_idx ON proposal_targets(memory_id, revision);
CREATE INDEX job_work_idx ON embedding_jobs(status, retry_at, lease_until);
