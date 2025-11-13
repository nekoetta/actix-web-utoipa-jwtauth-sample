-- Rollback: Remove performance indexes
DROP INDEX IF EXISTS idx_customer_categories_name;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_employee_number;
