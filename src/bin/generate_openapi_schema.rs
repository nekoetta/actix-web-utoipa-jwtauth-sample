use rust_api::swagger::generate_openapi_schema;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_openapi_schema()?;
    Ok(())
}
