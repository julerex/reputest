-- Create a database function to refresh all materialized views
-- This function uses SECURITY DEFINER to run with the privileges of the function owner,
-- allowing the reputest-rust-app user to refresh views owned by another role.
--
-- INSTRUCTIONS:
-- 1. Connect to your database as a superuser (postgres) or the owner of the materialized views
-- 2. Run this script to create the function
-- 3. The function will be owned by the user who creates it (should have permission to refresh the views)
-- 4. Grant EXECUTE permission to reputest-rust-app user
--
-- Usage from Rust code:
--   sqlx::query("SELECT refresh_all_materialized_views()").execute(pool).await?;

-- Drop the function if it already exists (for idempotency)
DROP FUNCTION IF EXISTS refresh_all_materialized_views();

-- Create the function with SECURITY DEFINER
-- This means the function runs with the privileges of the user who created it,
-- not the user who calls it
CREATE OR REPLACE FUNCTION refresh_all_materialized_views()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    start_time TIMESTAMP WITH TIME ZONE;
    elapsed_ms INTEGER;
BEGIN
    -- Refresh degree 1 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_one;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (1, NOW(), elapsed_ms);

    -- Refresh degree 2 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_two;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (2, NOW(), elapsed_ms);

    -- Refresh degree 3 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_three;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (3, NOW(), elapsed_ms);

    -- Refresh degree 4 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_four;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (4, NOW(), elapsed_ms);

    -- Refresh degree 5 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_five;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (5, NOW(), elapsed_ms);

    -- Refresh degree 6 view
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_good_vibes_degree_six;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (6, NOW(), elapsed_ms);

    -- Refresh combined view (record with degree=NULL)
    start_time := clock_timestamp();
    REFRESH MATERIALIZED VIEW view_all_good_vibes_degrees;
    elapsed_ms := EXTRACT(EPOCH FROM (clock_timestamp() - start_time) * 1000)::INTEGER;
    INSERT INTO vibe_materialize_time (degree, refresh_time, time_taken_ms)
    VALUES (NULL, NOW(), elapsed_ms);
END;
$$;

-- Grant execute permission to the application user
-- Replace 'reputest-rust-app' with your actual application database username if different
GRANT EXECUTE ON FUNCTION refresh_all_materialized_views() TO "reputest-rust-app";

-- Add a comment explaining the function
COMMENT ON FUNCTION refresh_all_materialized_views() IS 
'Refreshes all materialized views (degree 1-6 and combined view) and records timing metrics. '
'Uses SECURITY DEFINER to allow the application user to refresh views owned by another role.';





