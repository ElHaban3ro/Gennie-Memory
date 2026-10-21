CREATE TABLE dashboard_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id TEXT,
    query_key TEXT NOT NULL,
    query TEXT NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    entity_id TEXT,
    as_of TEXT NOT NULL,
    query_mode TEXT NOT NULL,
    candidates_json TEXT NOT NULL,
    decisions_json TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('matches', 'none', 'missing')),
    note TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX dashboard_reviews_query_idx ON dashboard_reviews(query_key, id DESC);
CREATE INDEX dashboard_reviews_case_idx ON dashboard_reviews(case_id, id DESC);
