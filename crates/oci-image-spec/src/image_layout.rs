use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ImageLayout {
    /// version of the OCI Image Layout (in the oci-layout file)
    #[serde(rename = "imageLayoutVersion")]
    pub image_layout_version: String,
}
