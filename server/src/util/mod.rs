use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};

// First, create a wrapper type for DateTime<Utc>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeWrapper(pub chrono::DateTime<chrono::Utc>);

// Implement JsonSchema for the wrapper
impl JsonSchema for DateTimeWrapper {
    fn schema_name() -> String {
        "DateTime".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("date-time".to_string()),
            ..Default::default()
        };

        Schema::Object(schema)
    }
}
