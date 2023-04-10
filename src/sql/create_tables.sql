CREATE TABLE IF NOT EXISTS "office_location" (
    instant TEXT  NOT NULL,
    location TEXT NOT NULL,
    PRIMARY KEY (instant, location)
);