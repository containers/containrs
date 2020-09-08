use crate::criapi::ImageSpec;
use crate::criapi::{self, image_service_server::ImageService};
use anyhow::Result;
use bytes::Bytes;
use oci_registry_client::{
    blob::Blob, manifest::Digest, manifest::Manifest, DockerRegistryClientV2,
};
use prost::Message;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::stream::Stream;
use tonic::{Request, Response, Status};

#[derive(thiserror::Error, Debug)]
pub enum ImageError {
    #[error("image database error: {0}")]
    DatabaseError(#[from] sled::Error),
    #[error("decode error, database likely corrupted: {0}")]
    DecodeError(#[from] prost::DecodeError),
}

impl From<ImageError> for Status {
    fn from(_: ImageError) -> Self {
        todo!()
    }
}

pub struct MyImage {
    database: sled::Db,
    layers: PathBuf,

    registries: Vec<DockerRegistryClientV2>,
    // TODO: prevent pulling same image simultaneously
    // pull_progress: RwLock<HashMap<ImageSpec, Mutex<()>>>,
}

fn blob_to_stream(mut blob: Blob) -> impl Stream<Item = tokio::io::Result<Bytes>> {
    async_stream::try_stream! {
        while let Some(bytes) = blob.chunk().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))? {
            yield bytes
        }
    }
}

impl MyImage {
    /// Open directory as image storage
    pub async fn open(image_storage: &Path) -> Result<Self> {
        let mut db_path = image_storage.to_owned();
        db_path.push("images.db");
        let mut layers = image_storage.to_owned();
        layers.push("layers");
        tokio::fs::create_dir_all(&layers).await?;
        Ok(MyImage {
            database: sled::open(db_path)?,
            layers,
            registries: vec![DockerRegistryClientV2::new(
                "registry.docker.io",
                "https://registry-1.docker.io",
                "https://auth.docker.io/token",
            )],
        })
    }

    /// List already pulled images
    /// TODO: use passed spec
    pub fn list_images(&self, _spec: Option<&ImageSpec>) -> Result<Vec<criapi::Image>, ImageError> {
        use std::io::Cursor;

        let mut k = sled::IVec::default();
        let mut out = vec![];
        while let Some((ik, v)) = self.database.get_gt(&k)? {
            k = ik.clone();
            out.push(criapi::Image::decode(&mut Cursor::new(&v))?);
        }
        Ok(out)
    }

    pub async fn download_layer(
        &self,
        registry: &DockerRegistryClientV2,
        image: &str,
        digest: &Digest,
    ) -> Result<()> {
        let mut out_dir = self.layers.clone();
        out_dir.push(format!("{}.tmp", digest));

        // TODO: lock by digest
        let blob = registry.blob(image, digest).await?;
        let blob_stream = blob_to_stream(blob);
        let blob_reader = tokio::io::stream_reader(blob_stream);
        let deflate = Box::pin(async_compression::tokio_02::bufread::GzipDecoder::new(
            blob_reader,
        ));
        let mut tar = tokio_tar::Archive::new(deflate);

        tar.unpack(out_dir).await?;

        Ok(())
    }

    pub async fn pull_image(&self, spec: &ImageSpec) -> Result<Option<Manifest>> {
        // TODO: handle registry name in image name (I.e docker.io/redis:latest)
        let parts = spec.image.split(':').collect::<Vec<&str>>();
        assert_eq!(parts.len(), 2, "bad name format");
        let image_name = parts[0];
        let reference = parts[1];
        for registry in &self.registries {
            let response = registry.auth("repository", image_name, "pull").await;
            let mut registry = registry.clone();
            if let Ok(token) = response {
                registry.set_auth_token(Some(token));
            }

            let manifests = match registry.list_manifests(image_name, reference).await {
                Ok(m) => m,
                // TODO: abort on error
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            };
            let mut found_manifest = None;
            for manifest in manifests.manifests {
                // TODO: check platform with passed annotations
                let manifest = registry
                    .manifest(image_name, &manifest.digest.to_string())
                    .await?;
                found_manifest.replace(manifest);
            }
            let manifest = if let Some(m) = found_manifest {
                m
            } else {
                return Ok(None);
            };

            // TODO: report progress in ImageStatusResponse info field
            // TODO: download in parallel
            for layer in manifest.layers {
                self.download_layer(&registry, image_name, &layer.digest)
                    .await?;
            }
        }
        Ok(None)
    }
}

#[tonic::async_trait]
impl ImageService for MyImage {
    async fn list_images(
        &self,
        request: Request<criapi::ListImagesRequest>,
    ) -> Result<Response<criapi::ListImagesResponse>, Status> {
        let request = request.into_inner();
        let images = self.list_images(request.filter.as_ref().and_then(|f| f.image.as_ref()))?;
        let resp = criapi::ListImagesResponse { images };
        Ok(Response::new(resp))
    }

    async fn pull_image(
        &self,
        request: Request<criapi::PullImageRequest>,
    ) -> Result<Response<criapi::PullImageResponse>, Status> {
        let request = request.into_inner();
        if let Some(spec) = request.image {
            self.pull_image(&spec).await.expect("pull");
        }

        let resp = criapi::PullImageResponse {
            image_ref: "some_image".into(),
        };
        Ok(Response::new(resp))
    }

    async fn image_status(
        &self,
        _request: Request<criapi::ImageStatusRequest>,
    ) -> Result<Response<criapi::ImageStatusResponse>, Status> {
        let resp = criapi::ImageStatusResponse {
            image: None,
            info: HashMap::new(),
        };
        Ok(Response::new(resp))
    }

    async fn remove_image(
        &self,
        _request: Request<criapi::RemoveImageRequest>,
    ) -> Result<Response<criapi::RemoveImageResponse>, Status> {
        let resp = criapi::RemoveImageResponse {};
        Ok(Response::new(resp))
    }

    async fn image_fs_info(
        &self,
        _request: Request<criapi::ImageFsInfoRequest>,
    ) -> Result<Response<criapi::ImageFsInfoResponse>, Status> {
        let resp = criapi::ImageFsInfoResponse {
            image_filesystems: Vec::new(),
        };
        Ok(Response::new(resp))
    }
}

#[cfg(test)]
pub mod tests {
    use super::MyImage;
    use crate::criapi::ImageSpec;
    use anyhow::{Context, Result};

    async fn create_tmp_image_service() -> Result<(tempfile::TempDir, MyImage)> {
        let tempdir = tempfile::tempdir().context("tempdir")?;
        let image_service = MyImage::open(tempdir.path())
            .await
            .context("image service open")?;
        Ok((tempdir, image_service))
    }

    #[tokio::test]
    pub async fn list_images() -> Result<()> {
        let (_tempdir, image_service) = create_tmp_image_service().await?;

        assert!(image_service.list_images(None)?.is_empty());

        Ok(())
    }

    #[tokio::test]
    pub async fn find_manifest() -> Result<()> {
        let (_tempdir, image_service) = create_tmp_image_service().await?;

        eprintln!("{:?}", _tempdir);

        assert!(image_service
            .pull_image(&ImageSpec {
                image: "library/redis:latest".into(),
                ..Default::default()
            })
            .await?
            .is_some());

        Ok(())
    }
}
