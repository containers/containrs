use serde::{Deserialize, Serialize};

use crate::{content_descriptor::ContentDescriptor, defs::Annotations};
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ImageManifest {
    pub annotations: Option<Annotations>,
    pub config: ContentDescriptor,
    pub layers: Vec<ContentDescriptor>,
    /// This field specifies the image manifest schema version as an integer
    #[serde(rename = "schemaVersion")]
    pub schema_version: i64,
}
