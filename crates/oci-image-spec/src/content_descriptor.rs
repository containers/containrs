use crate::defs::{Annotations, Digest, MediaType, Url};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ContentDescriptor {
    pub annotations: Option<Annotations>,
    /// the cryptographic checksum digest of the object, in the pattern '<algorithm>:<encoded>'
    pub digest: Digest,
    /// the mediatype of the referenced object
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,
    /// the size in bytes of the referenced object
    pub size: i64,
    /// a list of urls from which this object may be downloaded
    pub urls: Option<Vec<Url>>,
}
