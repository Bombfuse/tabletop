-- Migration: create "actions" tables: actions, attacks, interactions.
--
-- Notes:
-- - `actions` is the parent table.
-- - `attacks` and `interactions` are (optional) 1:1 extensions of an action, keyed by `action_id`.
-- - Enums are stored as TEXT with CHECK constraints.
-- - We mirror the timestamp/updated_at trigger style used in earlier migrations.

CREATE TABLE IF NOT EXISTS actions (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    name               TEXT NOT NULL UNIQUE,
    action_point_cost  INTEGER NOT NULL,
    action_type        TEXT NOT NULL,
    text               TEXT NOT NULL,

    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    CHECK (length(trim(name)) > 0),
    CHECK (length(trim(text)) > 0),
    CHECK (action_point_cost >= 0),
    CHECK (action_type IN ('Interaction', 'Attack'))
);

CREATE INDEX IF NOT EXISTS idx_actions_name ON actions(name);

CREATE TRIGGER IF NOT EXISTS trg_actions_updated_at
AFTER UPDATE ON actions
FOR EACH ROW
BEGIN
    UPDATE actions
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS attacks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    action_id    INTEGER NOT NULL UNIQUE,

    damage       INTEGER NOT NULL,
    damage_type  TEXT NOT NULL,
    skill        TEXT NOT NULL,
    target       INTEGER NOT NULL,
    range        INTEGER NOT NULL,

    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE,

    CHECK (damage >= 0),
    CHECK (range >= 0),
    CHECK (target BETWEEN 1 AND 14),
    CHECK (damage_type IN ('Arcane', 'Physical')),
    CHECK (skill IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility'))
);

CREATE INDEX IF NOT EXISTS idx_attacks_action_id ON attacks(action_id);

CREATE TRIGGER IF NOT EXISTS trg_attacks_updated_at
AFTER UPDATE ON attacks
FOR EACH ROW
BEGIN
    UPDATE attacks
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS interactions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    action_id  INTEGER NOT NULL UNIQUE,

    range      INTEGER NOT NULL,
    skill      TEXT NOT NULL,
    target     INTEGER NULL,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),

    FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE,

    CHECK (range >= 0),
    CHECK (target IS NULL OR target BETWEEN 1 AND 14),
    CHECK (skill IN ('Strength', 'Focus', 'Intelligence', 'Knowledge', 'Agility'))
);

CREATE INDEX IF NOT EXISTS idx_interactions_action_id ON interactions(action_id);

CREATE TRIGGER IF NOT EXISTS trg_interactions_updated_at
AFTER UPDATE ON interactions
FOR EACH ROW
BEGIN
    UPDATE interactions
    SET updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    WHERE id = OLD.id;
END;
