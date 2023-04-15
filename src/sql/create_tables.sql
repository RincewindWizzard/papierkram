BEGIN;

CREATE TABLE IF NOT EXISTS "office_location" (
    instant TEXT  NOT NULL,
    location TEXT NOT NULL,
    PRIMARY KEY (instant, location)
);

CREATE TABLE IF NOT EXISTS time_entries (
  id INTEGER PRIMARY KEY NOT NULL,
  description TEXT,
  start DATETIME NOT NULL,
  stop DATETIME,
  project_id INTEGER,
  workspace_id INTEGER
);

CREATE TABLE IF NOT EXISTS expected_time (
  date DATETIME PRIMARY KEY NOT NULL,
  duration INTEGER NOT NULL
);

CREATE VIEW worked_time_per_day AS SELECT
    DATE(start) as date,
    SUM(
        CAST(
            (julianday(datetime(stop)) - julianday(datetime(start))) * 86400
        AS INTEGER)
    ) AS actual_duration
FROM time_entries
GROUP BY DATE(start);

COMMIT;