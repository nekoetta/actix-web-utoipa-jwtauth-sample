use diesel::prelude::*;
use utoipa::ToSchema;
use tracing::instrument;
use crate::DbConnection;
use super::User;
use crate::schema::users::dsl;

#[derive(Debug, Insertable, ToSchema)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    login_id: &'a str,
    employee_number: Option<i32>,
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
    email: Option<&'a str>,
    gecos: Option<&'a str>
}

#[instrument(skip(conn), fields(db.operation = "insert_user", db.user = %uid))]
pub fn insert_new_user(conn: &mut DbConnection, uid: String,
        employee_number: Option<i32>,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
        gecos: Option<String>
    ) -> Result<User, crate::errors::ServiceError> {
        use crate::metrics::{DbMetrics, DurationTimer};
        use crate::traits::IntoValidator;
        use validator::Validate;

        // Requirements: 12.5 - Database metrics collection
        let timer = DurationTimer::new();
        DbMetrics::record_query("insert_user");

        // Create temporary user for validation
        // Requirements: 11.2 - Input validation
        let temp_user = User {
            id: 0, // Temporary ID for validation
            login_id: uid.clone(),
            employee_number,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            email: email.clone(),
            gecos: gecos.clone(),
        };

        // Validate user data before insertion
        temp_user.validator()
            .validate()
            .map_err(|e| {
                tracing::warn!(error = ?e, "User validation failed");
                crate::errors::ServiceError::ValidationError { value: e }
            })?;

        // Create insertion model
        let new_user = NewUser {
            login_id: &uid,
            employee_number,
            first_name: first_name.as_ref().map(|s| s.as_ref()),
            last_name: last_name.as_ref().map(|s| s.as_ref()),
            email: email.as_ref().map(|s| s.as_ref()),
            gecos: gecos.as_ref().map(|s| s.as_ref()),
        };

        // normal diesel operations
        let user = diesel::insert_into(dsl::users)
            .values(&new_user)
            .get_result(conn)
            .map_err(|e| {
                tracing::error!(error = ?e, "Database error during user insertion");
                crate::errors::ServiceError::InternalServerError
            })?;

        // Record query duration
        DbMetrics::record_duration("insert_user", timer.elapsed_secs());

        Ok(user)
    }

#[instrument(skip(conn), fields(db.operation = "find_user", db.user_id = %user_id))]
pub fn find_user(
    conn: &mut DbConnection,
    user_id: i32
) -> diesel::QueryResult<User> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("find_user");

    let user = dsl::users
        .find(&user_id)
        .first(conn)?;

    // Record query duration
    DbMetrics::record_duration("find_user", timer.elapsed_secs());

    Ok(user)
}

#[instrument(skip(conn), fields(db.operation = "search_user", db.user = %login_id))]
pub fn search_user(
    conn: &mut DbConnection,
    login_id: &str
) -> diesel::QueryResult<Vec<User>> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("search_user");

    let results = dsl::users
        .filter(dsl::login_id.eq(login_id))
        .load::<User>(conn)?;

    // Record query duration
    DbMetrics::record_duration("search_user", timer.elapsed_secs());

    Ok(results)
}

#[instrument(skip(conn), fields(db.operation = "all_users"))]
pub fn all_user(
    conn: &mut DbConnection
) -> diesel::QueryResult<Vec<User>> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("all_users");

    let results = dsl::users
        .load::<User>(conn)?;

    // Record query duration
    DbMetrics::record_duration("all_users", timer.elapsed_secs());

    Ok(results)
}

#[instrument(skip(conn), fields(db.operation = "all_users_paginated", page = %page, per_page = %per_page))]
pub fn all_user_paginated(
    conn: &mut DbConnection,
    page: i64,
    per_page: i64
) -> diesel::QueryResult<Vec<User>> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    // Requirements: 11.1 - Pagination implementation
    let timer = DurationTimer::new();
    DbMetrics::record_query("all_users_paginated");

    let offset = (page - 1) * per_page;
    let results = dsl::users
        .limit(per_page)
        .offset(offset)
        .load::<User>(conn)?;

    // Record query duration
    DbMetrics::record_duration("all_users_paginated", timer.elapsed_secs());

    Ok(results)
}
