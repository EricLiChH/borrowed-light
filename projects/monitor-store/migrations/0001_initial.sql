CREATE TABLE targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL
);

CREATE TABLE check_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id),
    status INTEGER,
    failure_kind TEXT,
    reason TEXT
);

CREATE INDEX check_results_target_id_id
    ON check_results (target_id, id DESC);
