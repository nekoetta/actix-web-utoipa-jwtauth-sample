CREATE TABLE
    customer_categories (
        id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        name VARCHAR(255) NOT NULL
    )
