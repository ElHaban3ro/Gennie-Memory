CREATE TABLE embedding_spaces (
    space_id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    model_sha256 TEXT NOT NULL CHECK (length(model_sha256) = 64),
    tokenizer_sha256 TEXT NOT NULL CHECK (length(tokenizer_sha256) = 64),
    procedure TEXT NOT NULL,
    dimensions INTEGER NOT NULL CHECK (dimensions > 0),
    vector_format TEXT NOT NULL CHECK (vector_format = 'json-f32-v1')
);

CREATE TABLE memory_embeddings (
    memory_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    space_id TEXT NOT NULL REFERENCES embedding_spaces(space_id),
    vector_json TEXT NOT NULL,
    norm REAL NOT NULL CHECK (norm > 0),
    PRIMARY KEY (memory_id, revision, space_id),
    FOREIGN KEY (memory_id, revision) REFERENCES memory_revisions(memory_id, revision) ON DELETE CASCADE
);

CREATE INDEX memory_embeddings_space_idx ON memory_embeddings(space_id, memory_id, revision);
