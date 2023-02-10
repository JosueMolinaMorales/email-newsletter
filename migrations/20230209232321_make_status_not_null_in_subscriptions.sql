-- Add migration script here
-- Wrap the whole migration in a transaction to make sure
-- it succeeds or fails atomically.
BEGIN;
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;