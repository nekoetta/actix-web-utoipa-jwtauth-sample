use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use tracing::instrument;
use crate::{DbConnection, errors::ServiceError, models::validate, traits::IntoValidator};
use super::{CustomerCategory, CategoryValidator};
use crate::schema::customer_categories::dsl;

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::customer_categories)]
struct NewCategory<'a> {
    pub name: &'a str,
}

impl<'a> IntoValidator<CategoryValidator> for NewCategory<'a> {
    fn validator(&self) -> CategoryValidator {
        CategoryValidator { name: self.name.to_string() }
    }
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct NewCategoryBody {
    pub name: String
}

#[instrument(skip(conn), fields(db.operation = "insert_category", db.category = %name))]
pub fn insert_new_category(conn: &mut DbConnection, name: &str) -> Result<CustomerCategory, ServiceError> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("insert_category");

    let new_category = NewCategory {
        name
    };

    validate::<CategoryValidator>(&new_category)?;

    let category = diesel::insert_into(dsl::customer_categories)
        .values(&new_category)
        .get_result(conn)
        .map_err(|_e| ServiceError::InternalServerError)?;

    // Record query duration
    DbMetrics::record_duration("insert_category", timer.elapsed_secs());

    Ok(category)
}

#[instrument(skip(conn), fields(db.operation = "update_category", db.category_id = %id, db.category = %name))]
pub fn update_category(conn: &mut DbConnection, id: i32, name: &str) -> Result<CustomerCategory, ServiceError> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("update_category");

    let customer_category = CustomerCategory {
        id,
        name: name.to_string()
    };

    validate::<CategoryValidator>(&customer_category)?;

    let category = diesel::update(&customer_category)
        .set(dsl::name.eq(name))
        .get_result(conn)
        .map_err(|_e| ServiceError::InternalServerError)?;

    // Record query duration
    DbMetrics::record_duration("update_category", timer.elapsed_secs());

    Ok(category)
}

#[instrument(skip(conn), fields(db.operation = "all_categories"))]
pub fn all_categories(
    conn: &mut DbConnection
) -> Result<Vec<CustomerCategory>, ServiceError> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("all_categories");

    let results = dsl::customer_categories
        .order(dsl::id.asc())
        .load::<CustomerCategory>(conn)
        .map_err(|_e| ServiceError::InternalServerError)?;

    // Record query duration
    DbMetrics::record_duration("all_categories", timer.elapsed_secs());

    Ok(results)
}

#[instrument(skip(conn), fields(db.operation = "all_categories_paginated", page = %page, per_page = %per_page))]
pub fn all_categories_paginated(
    conn: &mut DbConnection,
    page: i64,
    per_page: i64
) -> Result<Vec<CustomerCategory>, ServiceError> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    // Requirements: 11.1 - Pagination implementation
    let timer = DurationTimer::new();
    DbMetrics::record_query("all_categories_paginated");

    let offset = (page - 1) * per_page;
    let results = dsl::customer_categories
        .order(dsl::id.asc())
        .limit(per_page)
        .offset(offset)
        .load::<CustomerCategory>(conn)
        .map_err(|_e| ServiceError::InternalServerError)?;

    // Record query duration
    DbMetrics::record_duration("all_categories_paginated", timer.elapsed_secs());

    Ok(results)
}

#[instrument(skip(conn), fields(db.operation = "get_category", db.category_id = %id))]
pub fn get_category(
    conn: &mut DbConnection,
    id: i32
) -> Result<CustomerCategory, ServiceError> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("get_category");

    let result = dsl::customer_categories
        .find(id)
        .get_result(conn)
        .map_err(|_e| ServiceError::InternalServerError)?;

    // Record query duration
    DbMetrics::record_duration("get_category", timer.elapsed_secs());

    Ok(result)
}

#[instrument(skip(conn), fields(db.operation = "destroy_category", db.category_id = %id))]
pub fn destroy_category(
    conn: &mut DbConnection,
    id: i32
) -> Result<CustomerCategory, ServiceError> {
    use crate::metrics::{DbMetrics, DurationTimer};

    // Requirements: 12.5 - Database metrics collection
    let timer = DurationTimer::new();
    DbMetrics::record_query("destroy_category");

    let result = diesel::delete(dsl::customer_categories)
        .filter(dsl::id.eq(id))
        .get_result(conn)
        .map_err(|_e| ServiceError::InternalServerError)?;

    // Record query duration
    DbMetrics::record_duration("destroy_category", timer.elapsed_secs());

    Ok(result)
}


#[cfg(test)]
mod tests {
    use validator::{ValidationErrors, ValidationError};
    use super::*;
    use crate::create_connection_pool;

    #[test]
    fn insert_customer_category_test() {
        let pool = create_connection_pool();
        let mut conn = pool.get().unwrap();
        let test_name = "test";

        conn.test_transaction::<_, ServiceError, _>(|conn| {

            let inserted_category = insert_new_category(conn, test_name).unwrap();
            assert_eq!(inserted_category.name, test_name);
            Ok(())
        })
    }

    #[test]
    fn update_customer_category_test() {
        let pool = create_connection_pool();
        let mut conn = pool.get().unwrap();
        
        conn.test_transaction::<_, ServiceError, _>(|conn| {
            
            let test_name = "test";
            let inserted_category = insert_new_category(conn, test_name).unwrap();
            
            let update_name = "update";
            let updated_category = update_category(conn, inserted_category.id, update_name).unwrap();
            assert_eq!(updated_category, CustomerCategory {id: inserted_category.id, name: update_name.to_string()});

            Ok(())
        })
    }

    #[test]
    fn customer_category_validation_error_test() {
        let pool = create_connection_pool();
        let mut conn = pool.get().unwrap();
        let test_name = std::iter::repeat('a').take(256).collect::<String>();

        conn.test_transaction::<_, ServiceError, _>(|conn| {

            let error = insert_new_category(conn, test_name.as_str()).unwrap_err();
            let mut validation_errors = ValidationErrors::new();
            let mut validation_error = ValidationError::new("length");
            validation_error.message = Some("顧客分類は255文字以下で入力してください".into());
            validation_error.add_param("value".into(), &test_name);
            validation_error.add_param("max".into(), &255);
            validation_errors.add("name", validation_error);
            assert_eq!(error, ServiceError::ValidationError{ value: validation_errors });
            Ok(())
        })
    }
}
