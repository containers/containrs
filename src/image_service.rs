use crate::criapi::{self, image_service_server::ImageService};
use anyhow::Result;
use prost::Message;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    images: PathBuf,
}

impl MyImage {
    /// Open directory as image storage
    pub fn open(image_storage: &Path) -> Result<Self> {
        let mut db_path = image_storage.to_owned();
        db_path.push("images.db");
        let mut images = image_storage.to_owned();
        images.push("images");
        Ok(MyImage {
            database: sled::open(db_path)?,
            images,
        })
    }

    /// List already pulled images
    /// TODO: use passed spec
    pub fn list_images(
        &self,
        _spec: Option<&criapi::ImageSpec>,
    ) -> Result<Vec<criapi::Image>, ImageError> {
        use std::io::Cursor;

        let mut k = sled::IVec::default();
        let mut out = vec![];
        while let Some((ik, v)) = self.database.get_gt(&k)? {
            k = ik.clone();
            out.push(criapi::Image::decode(&mut Cursor::new(&v))?);
        }
        Ok(out)
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
        _request: Request<criapi::PullImageRequest>,
    ) -> Result<Response<criapi::PullImageResponse>, Status> {
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
    use anyhow::{Context, Result};

    fn create_tmp_image_service() -> Result<(tempfile::TempDir, MyImage)> {
        let tempdir = tempfile::tempdir().context("tempdir")?;
        let image_service = MyImage::open(tempdir.path()).context("image service open")?;
        Ok((tempdir, image_service))
    }

    #[tokio::test]
    pub async fn list_images() -> Result<()> {
        let (_tempdir, image_service) = create_tmp_image_service()?;

        assert!(image_service.list_images(None)?.is_empty());

        Ok(())
    }
}
