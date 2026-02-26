CREATE TABLE timeline_days (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    day_date date NOT NULL UNIQUE,

    year int NOT NULL,
    month int NOT NULL,

    total_count int NOT NULL,

    min_sort_date timestamptz,
    max_sort_date timestamptz,

    created_at timestamptz DEFAULT now()
);

CREATE INDEX idx_timeline_days_day_desc
ON timeline_days (day_date DESC);

CREATE INDEX idx_timeline_days_year
ON timeline_days (year);

CREATE INDEX idx_timeline_days_year_day_desc
ON timeline_days (year, day_date DESC);

INSERT INTO timeline_days (
    day_date,
    year,
    month,
    total_count,
    min_sort_date,
    max_sort_date
)
SELECT
    p.day_date,
    EXTRACT(YEAR FROM p.day_date)::int,
    EXTRACT(MONTH FROM p.day_date)::int,
    COUNT(*) AS total_count,
    MIN(p.sort_date),
    MAX(p.sort_date)
FROM photos p
WHERE p.day_date IS NOT NULL
GROUP BY p.day_date
ORDER BY p.day_date;

SELECT * from timeline_days order;

SELECT year, SUM(total_count) as photo_count
FROM timeline_days
GROUP BY year
ORDER BY year DESC;

SELECT day_date
FROM timeline_days
WHERE year = 2025
ORDER BY day_date DESC
LIMIT 1;