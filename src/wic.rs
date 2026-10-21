//! Image decoding through Windows Imaging Component, for formats the `image` crate can't read.
//! Mainly HEIC from iPhone (needs the "HEIF Image Extensions" + "HEVC Video Extensions" from the
//! Microsoft Store, which most Windows 11 PCs already have).

use image::RgbaImage;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICImagingFactory, WICConvertBitmapSource,
    WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};

pub fn decode(data: &[u8]) -> Result<RgbaImage, String> {
    unsafe {
        // Each HTTP request runs on its own thread; S_FALSE ("already initialized") is fine.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        let stream = factory.CreateStream().map_err(|e| e.to_string())?;
        stream.InitializeFromMemory(data).map_err(|e| e.to_string())?;
        let decoder = factory
            .CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)
            .map_err(|e| format!("Windows bu resmi cozemedi (HEIF/HEVC eklentisi?): {e}"))?;
        let frame = decoder.GetFrame(0).map_err(|e| e.to_string())?;
        let rgba = WICConvertBitmapSource(&GUID_WICPixelFormat32bppRGBA, &frame).map_err(|e| e.to_string())?;
        let (mut w, mut h) = (0u32, 0u32);
        rgba.GetSize(&mut w, &mut h).map_err(|e| e.to_string())?;
        let mut pixels = vec![0u8; w as usize * h as usize * 4];
        rgba.CopyPixels(std::ptr::null(), w * 4, &mut pixels).map_err(|e| e.to_string())?;
        RgbaImage::from_raw(w, h, pixels).ok_or_else(|| "bozuk resim boyutu".to_string())
    }
}
