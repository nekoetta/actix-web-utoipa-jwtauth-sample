// @generated automatically by Diesel CLI.

diesel::table! {
    customer_categories (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        login_id -> Varchar,
        employee_number -> Nullable<Int4>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        gecos -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(customer_categories, users,);
