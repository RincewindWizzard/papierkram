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

CREATE TABLE IF NOT EXISTS expected_duration (
  date DATETIME PRIMARY KEY NOT NULL,
  duration INTEGER NOT NULL
);


DROP VIEW IF EXISTS timesheet; -- "OR REPLACE"
CREATE VIEW timesheet AS WITH
    -- group all time entries per their dates and sum the durations
    worked_time_per_day AS (
        SELECT
            DATE(start) as date,
            SUM(
                CAST(
                    (julianday(datetime(IFNULL(stop, datetime('now')))) - julianday(datetime(start))) * 86400
                AS INTEGER)
            ) AS actual_duration
        FROM time_entries
        GROUP BY DATE(start)
    ),
    -- calculate the delta of actual time worked to expected work time
    timesheet_delta AS (
        SELECT
            worked_time_per_day.date as date,
            actual_duration,
            expected_duration.duration AS expected_duration,
            (actual_duration - expected_duration.duration) AS delta,

            -- beginning of the 'typical' workday
            "08:00:00" as normalized_start_of_business,
            -- if started at the 'typical' time, done the obligatory breaks, which time would we currently?
            -- 28800 is 08:00
            time(28800 + actual_duration  +
                CASE -- after 6 hours there needs to be a break of 45 minutes
                    WHEN actual_duration > 21600  THEN 2700
                    ELSE 0
                END,
            'unixepoch') as normalized_end_of_business
        FROM worked_time_per_day
        LEFT JOIN expected_duration
        ON worked_time_per_day.date = expected_duration.date
    ),
    events_per_day AS (
	SELECT
       	    DATE(instant) as date,
	    location as event_name
	FROM office_location
	GROUP BY DATE(instant), location
    )
SELECT
    date,
    actual_duration,
    expected_duration,
    delta,
    ( -- sum all deltas before this date
        SELECT SUM(delta) FROM timesheet_delta AS saldo_table WHERE timesheet.date >= saldo_table.date
    ) AS saldo,
    normalized_start_of_business,
    normalized_end_of_business,
    (
        SELECT GROUP_CONCAT(event_name, ", ")
        FROM events_per_day
        GROUP BY events_per_day.date
        HAVING events_per_day.date = timesheet.date
    ) AS events
FROM timesheet_delta AS timesheet
ORDER BY timesheet.date;

COMMIT;