-- Requirements: 11.4 - Add indexes to frequently searched columns
-- Add index on customer_categories.name for faster category searches
CREATE INDEX idx_customer_categories_name ON customer_categories(name);

-- Add index on users.email for faster email lookups
CREATE INDEX idx_users_email ON users(email);

-- Add index on users.employee_number for faster employee number searches
CREATE INDEX idx_users_employee_number ON users(employee_number);
