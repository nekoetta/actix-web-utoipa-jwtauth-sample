use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub mod usecases;

#[derive(Clone, Queryable, Deserialize, Serialize, ToSchema, Debug)]
pub struct User {
    pub id: i32,
    pub login_id: String,
    pub employee_number: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub gecos: Option<String>
}

use validator::Validate;

#[derive(Debug, Validate)]
pub struct UserValidator {
    #[validate(length(min = 1, max = 255, message = "ログインIDは1文字以上255文字以下で入力してください"))]
    pub login_id: String,
    
    #[validate(range(min = 1, message = "社員番号は1以上の値を入力してください"))]
    pub employee_number: Option<i32>,
    
    #[validate(length(max = 255, message = "名は255文字以下で入力してください"))]
    pub first_name: Option<String>,
    
    #[validate(length(max = 255, message = "姓は255文字以下で入力してください"))]
    pub last_name: Option<String>,
    
    #[validate(email(message = "有効なメールアドレスを入力してください"))]
    #[validate(length(max = 255, message = "メールアドレスは255文字以下で入力してください"))]
    pub email: Option<String>,
    
    #[validate(length(max = 500, message = "gecosは500文字以下で入力してください"))]
    pub gecos: Option<String>,
}

impl crate::traits::IntoValidator<UserValidator> for User {
    fn validator(&self) -> UserValidator {
        UserValidator {
            login_id: self.login_id.clone(),
            employee_number: self.employee_number,
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email: self.email.clone(),
            gecos: self.gecos.clone(),
        }
    }
}
