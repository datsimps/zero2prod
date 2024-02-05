-- SQL transacton
BEGIN;
    -- Backfill historical entries
    UPDATE subscriptions
    SET status = 'confirmed'
    -- Make status mandatory moving foward
    WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
