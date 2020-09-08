use crate::defs::{Digest, MediaType, Url};

use super::defs::Annotations;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ImageIndexItemManifestsPlatform {
    pub architecture: String,
    pub os: String,
    #[serde(rename = "os.features")]
    pub os_features: Option<Vec<String>>,
    #[serde(rename = "os.version")]
    pub os_version: Option<String>,
    pub variant: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ImageIndexItemManifests {
    pub annotations: Option<Annotations>,
    /// the cryptographic checksum digest of the object, in the pattern '<algorithm>:<encoded>'
    pub digest: Digest,
    /// the mediatype of the referenced object
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    pub platform: Option<ImageIndexItemManifestsPlatform>,
    /// the size in bytes of the referenced object
    pub size: i64,
    /// a list of urls from which this object may be downloaded
    pub urls: Option<Vec<Url>>,
}
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ImageIndex {
    pub annotations: Option<Annotations>,
    pub manifests: Vec<ImageIndexItemManifests>,
    /// This field specifies the image index schema version as an integer
    #[serde(rename = "schemaVersion")]
    pub schema_version: i64,
}
