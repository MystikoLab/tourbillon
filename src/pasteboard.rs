use std::path::PathBuf;

use objc2::{
    ClassType,
    rc::Retained,
    runtime::{AnyClass, ProtocolObject},
};
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSColor, NSColorSpace, NSPasteboard, NSPasteboardItem,
    NSPasteboardTypePNG, NSPasteboardTypeTIFF, NSPasteboardURLReadingFileURLsOnlyKey,
    NSPasteboardWriting,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSNumber, NSString, NSURL};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(PartialEq, Clone, Debug)]
pub enum PasteboardItem {
    Files(Vec<PathBuf>),
    Image(Vec<u8>),
    Text(String),
    Urls(String),
    Colour(Rgba),
}

impl PasteboardItem {
    pub fn get_current_pasteboard_item() -> Option<Self> {
        if let Some(files) = Self::read_files()
            && !files.is_empty()
        {
            Some(PasteboardItem::Files(files))
        } else if let Some(colour) = Self::read_colour() {
            Some(PasteboardItem::Colour(colour))
        } else if let Some(url) = Self::read_url() {
            Some(PasteboardItem::Urls(url))
        } else if let Some(image) = Self::read_image() {
            Some(PasteboardItem::Image(image))
        } else if let Some(text) = Self::read_string() {
            Some(PasteboardItem::Text(text))
        } else {
            None
        }
    }

    pub fn write_to_clipboard(&self) {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        let arr: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> = match self {
            PasteboardItem::Files(files) => files
                .iter()
                .filter_map(|path| NSURL::from_path(path, path.is_dir(), None))
                .map(ProtocolObject::from_retained)
                .collect(),
            PasteboardItem::Text(str) | PasteboardItem::Urls(str) => {
                vec![ProtocolObject::from_retained(NSString::from_str(str))]
            }
            PasteboardItem::Colour(clr) => {
                vec![ProtocolObject::from_retained(
                    NSColor::colorWithRed_green_blue_alpha(clr.r, clr.g, clr.b, clr.a),
                )]
            }
            PasteboardItem::Image(bytes) => {
                let item = NSPasteboardItem::new();
                let data = NSData::with_bytes(bytes.as_slice());
                unsafe { item.setData_forType(&data, NSPasteboardTypePNG) };
                vec![ProtocolObject::from_retained(item)]
            }
        };

        pasteboard.writeObjects(&NSArray::from_retained_slice(&arr));
    }

    fn read_string() -> Option<String> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let pasteboard_classes: Retained<NSArray<AnyClass>> =
            NSArray::from_slice(&[NSString::class()]);

        unsafe { pasteboard.readObjectsForClasses_options(&pasteboard_classes, None) }?
            .firstObject()?
            .downcast_ref::<NSString>()
            .map(|x| x.to_string())
    }

    fn read_image() -> Option<Vec<u8>> {
        let pasteboard = NSPasteboard::generalPasteboard();

        if let Some(data) = unsafe { pasteboard.dataForType(NSPasteboardTypePNG) } {
            return Some(data.to_vec());
        }

        let tiff = unsafe { pasteboard.dataForType(NSPasteboardTypeTIFF) }?;
        let rep = NSBitmapImageRep::imageRepWithData(&tiff)?;
        let png = unsafe {
            rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
        }?;

        Some(png.to_vec())
    }

    fn read_files() -> Option<Vec<PathBuf>> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let pasteboard_classes = NSArray::from_slice(&[NSURL::class()]);

        let key = unsafe { NSPasteboardURLReadingFileURLsOnlyKey };
        let yes = NSNumber::new_bool(true);
        let options = NSDictionary::from_slices(&[key], &[yes.as_ref()]);

        Some(unsafe {
            pasteboard
                .readObjectsForClasses_options(&pasteboard_classes, Some(&options))
                .iter()
                .filter_map(|arr| {
                    let url = arr.downcast_ref::<NSURL>()?;
                    Some(PathBuf::from(url.path()?.to_string()))
                })
                .collect()
        })
    }

    fn read_url() -> Option<String> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let pasteboard_classes: Retained<NSArray<AnyClass>> =
            NSArray::from_slice(&[NSURL::class()]);

        unsafe { pasteboard.readObjectsForClasses_options(&pasteboard_classes, None) }?
            .firstObject()?
            .downcast_ref::<NSURL>()
            .and_then(|x| {
                if x.isFileURL() {
                    None
                } else {
                    x.absoluteString().map(|s| s.to_string())
                }
            })
    }

    fn read_colour() -> Option<Rgba> {
        let pasteboard = NSPasteboard::generalPasteboard();
        let pasteboard_classes: Retained<NSArray<AnyClass>> =
            NSArray::from_slice(&[NSColor::class()]);

        unsafe { pasteboard.readObjectsForClasses_options(&pasteboard_classes, None) }?
            .firstObject()?
            .downcast_ref::<NSColor>()
            .and_then(|x| x.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace()))
            .map(|x| Rgba {
                r: x.redComponent(),
                g: x.greenComponent(),
                b: x.blueComponent(),
                a: x.alphaComponent(),
            })
    }
}
