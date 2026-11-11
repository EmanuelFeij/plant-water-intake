-- Sample plants for local development.
-- INSERT OR IGNORE skips slugs that already exist, so running this twice is safe.
-- last_watered_at is Unix seconds; NULL means never watered.

INSERT OR IGNORE INTO plants (slug, name, interval_days, last_watered_at) VALUES
    ('monstera',       'Monstera',          7,  unixepoch('now', '-9 days')),  -- overdue
    ('pothos-kitchen', 'Pothos (kitchen)',  5,  unixepoch('now', '-2 days')),  -- fine
    ('snake-plant',    'Snake plant',      14,  unixepoch('now', '-14 days')), -- due about now
    ('basil',          'Basil',             2,  NULL);                         -- never watered
