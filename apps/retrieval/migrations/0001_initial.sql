CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    current_revision INTEGER NOT NULL CHECK (current_revision > 0)
);

CREATE TABLE memory_revisions (
    memory_id TEXT NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    memory_type TEXT NOT NULL,
    original_text TEXT NOT NULL CHECK (length(trim(original_text)) > 0),
    summary TEXT NOT NULL CHECK (length(trim(summary)) > 0),
    state TEXT NOT NULL CHECK (state IN ('active', 'uncertain', 'expired', 'corrected')),
    valid_from TEXT,
    valid_until TEXT,
    PRIMARY KEY (memory_id, revision)
);

CREATE INDEX memory_scope_idx ON memories(scope_kind, scope_id, current_revision);
CREATE INDEX memory_type_idx ON memory_revisions(memory_type);

CREATE TABLE search_documents (
    doc_id INTEGER PRIMARY KEY,
    memory_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    original_text TEXT NOT NULL,
    summary TEXT NOT NULL,
    UNIQUE(memory_id, revision),
    FOREIGN KEY(memory_id, revision) REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
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
