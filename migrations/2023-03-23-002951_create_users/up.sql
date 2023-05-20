CREATE TABLE
    users (
        id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        login_id VARCHAR NOT NULL UNIQUE,
        employee_number INTEGER,
        first_name VARCHAR,
        last_name VARCHAR,
        email VARCHAR,
        gecos VARCHAR
    )
