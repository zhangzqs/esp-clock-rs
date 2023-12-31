use image::{imageops::FilterType, GenericImageView};
use poem_openapi::{
    payload::{Attachment, AttachmentType},
    OpenApi,
};

pub struct PhotoService;

#[OpenApi]
impl PhotoService {
    #[oai(path = "/photo", method = "get")]
    async fn photo(&self) -> Attachment<Vec<u8>> {
        let resp = reqwest::get("http://little-paimon.zzq:5000/img")
            .await
            .unwrap();
        let bytes = resp.bytes().await.unwrap();
        let img = image::load_from_memory(&bytes).unwrap();
        // 将图片缩放为 240x240
        let img = img.resize(240, 240, FilterType::Nearest);
        let mut bytes = Vec::new();
        // 写入一个字节的宽度和一个字节的高度
        bytes.push(img.width() as u8);
        bytes.push(img.height() as u8);
        // 依次写入原始图片像素数据，按照RGB888
        for (_, _, pixel) in img.pixels() {
            bytes.push(pixel.0[0]);
            bytes.push(pixel.0[1]);
            bytes.push(pixel.0[1]);
        }
        Attachment::new(bytes.to_vec()).attachment_type(AttachmentType::Inline)
    }
}
