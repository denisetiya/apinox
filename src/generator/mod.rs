pub mod hurl;
pub mod insomnia;
pub mod markdown;
pub mod openapi;
pub mod postman;
pub mod scalar;

use crate::schema::root::ApinoxSchema;
use anyhow::Result;

#[derive(Clone, PartialEq)]
pub enum OutputFormat {
    Postman,
    Openapi,
    Markdown,
    Scalar,
    Insomnia,
    Hurl,
}

pub fn generate(schema: &ApinoxSchema, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Postman => postman::generate(schema),
        OutputFormat::Openapi => openapi::generate(schema),
        OutputFormat::Markdown => markdown::generate(schema),
        OutputFormat::Scalar => scalar::generate(schema),
        OutputFormat::Insomnia => insomnia::generate(schema),
        OutputFormat::Hurl => hurl::generate(schema),
    }
}
