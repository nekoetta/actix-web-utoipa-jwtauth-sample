use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{schema::customer_categories, traits::IntoValidator};
use validator::Validate;

pub mod usecases;


#[derive(Validate)]
struct CategoryValidator{
    #[validate(length(max = 255, message="顧客分類は255文字以下で入力してください"))]
    pub name: String
}

#[derive(PartialEq, Clone, Queryable, Identifiable, Deserialize, Serialize, ToSchema, Debug)]
#[diesel(table_name=customer_categories)]
pub struct CustomerCategory {
    pub id: i32,
    pub name: String
}

impl IntoValidator<CategoryValidator> for CustomerCategory {
    fn validator(&self) -> CategoryValidator {
        CategoryValidator { name: self.name.clone() }
    }
}
