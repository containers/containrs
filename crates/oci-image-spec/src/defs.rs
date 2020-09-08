use std::collections::HashMap;

pub type Annotations = HashMap<String, String>;
/// the cryptographic checksum digest of the object, in the pattern '<algorithm>:<encoded>'
pub type Digest = String;
/// https://opencontainers.org/schema/image/descriptor/mediaType
pub type MediaType = String;
pub type Url = String;
