CREATE TABLE local_embedding_profiles (
    space_id TEXT PRIMARY KEY,
    profile_json TEXT NOT NULL
);
CREATE TABLE local_embedding_indexes (
    index_id TEXT PRIMARY KEY,
    space_id TEXT NOT NULL REFERENCES local_embedding_profiles(space_id),
    actor_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    manifest_sha256 TEXT NOT NULL,
    memory_count INTEGER NOT NULL CHECK (memory_count > 0),
    chunk_count INTEGER NOT NULL CHECK (chunk_count > 0),
    published_at TEXT NOT NULL
);
CREATE TABLE local_embedding_chunks (
    index_id TEXT NOT NULL REFERENCES local_embedding_indexes(index_id) ON DELETE CASCADE,
    memory_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    field TEXT NOT NULL CHECK (field IN ('original','summary')),
    ordinal INTEGER NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL CHECK (end_byte > start_byte),
    source_sha256 TEXT NOT NULL,
    chunk_sha256 TEXT NOT NULL,
    token_count INTEGER NOT NULL CHECK (token_count > 0),
    vector_blob BLOB NOT NULL,
    norm REAL NOT NULL CHECK (norm > 0),
    PRIMARY KEY (index_id,memory_id,revision,field,ordinal),
    FOREIGN KEY (memory_id,revision) REFERENCES memory_revisions(memory_id,revision)
);
CREATE INDEX local_embedding_chunk_memory ON local_embedding_chunks(index_id,memory_id,revision);
