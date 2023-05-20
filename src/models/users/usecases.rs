use diesel::prelude::*;
use utoipa::ToSchema;
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

pub fn insert_new_user(conn: &mut DbConnection, uid: String,
        employee_number: Option<i32>,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
        gecos: Option<String>
    ) -> diesel::QueryResult<User> {


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
            .expect("Error inserting person");

        Ok(user)
    }

pub fn find_user(
    conn: &mut DbConnection,
    user_id: i32
) -> diesel::QueryResult<User> {
    let user = dsl::users
        .find(&user_id)
        .first(conn)
        .expect("user not found");

    Ok(user)
}

pub fn search_user(
    conn: &mut DbConnection,
    login_id: &str
) -> diesel::QueryResult<Vec<User>> {
    let results = dsl::users
        .filter(dsl::login_id.eq(login_id))
        .load::<User>(conn)?;

    Ok(results)
}

pub fn all_user(
    conn: &mut DbConnection
) -> diesel::QueryResult<Vec<User>> {
    let results = dsl::users
        .load::<User>(conn)?;

    Ok(results)
}
