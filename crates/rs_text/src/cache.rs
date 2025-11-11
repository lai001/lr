use image::{GenericImage, GenericImageView};
use linebender_resource_handle::Blob;
use rs_pack::{rect::Rect, skyline::SkylineBinPack};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GlyphKey {
    pub font_name: String,
    pub character: char,
    pub size: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GlyphStoreInfo {
    pub page_index: usize,
    pub rect: Rect,
}

pub struct Atlas {
    packer: SkylineBinPack,
    data: image::GrayImage,
}

impl Atlas {
    pub fn data(&self) -> &image::GrayImage {
        &self.data
    }
}

pub struct FontInfo {
    font: fontdue::Font,
    data: Blob<u8>,
    glyphs: HashMap<u32, char>,
}

impl FontInfo {
    fn new(font: fontdue::Font, data: Blob<u8>) -> FontInfo {
        let glyphs = font
            .chars()
            .iter()
            .map(|(k, v)| (v.get() as u32, *k))
            .collect();
        FontInfo { font, data, glyphs }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct BlobID(u64);

impl From<u64> for BlobID {
    fn from(value: u64) -> BlobID {
        BlobID(value)
    }
}

pub struct FontCache {
    glyph_cache: HashMap<GlyphKey, GlyphStoreInfo>,
    atlas_cache: Vec<Atlas>,
    fonts_cache: HashMap<String, FontInfo>,
    font_names: HashMap<BlobID, String>,
    dirty_pages: HashSet<usize>,
}

impl Default for FontCache {
    fn default() -> FontCache {
        FontCache::new(1024, 2, 2)
    }
}

impl FontCache {
    pub fn new(glyph_capacity: usize, atlas_capacity: usize, fonts_capacity: usize) -> FontCache {
        let glyph_cache = HashMap::with_capacity(glyph_capacity);
        let atlas_cache = Vec::with_capacity(atlas_capacity);
        let fonts_cache = HashMap::with_capacity(fonts_capacity);
        let font_names = HashMap::with_capacity(fonts_capacity);
        FontCache {
            glyph_cache,
            atlas_cache,
            fonts_cache,
            dirty_pages: HashSet::new(),
            font_names,
        }
    }

    pub fn push_font(
        &mut self,
        name: Option<String>,
        font_blob: Blob<u8>,
    ) -> crate::error::Result<String> {
        let font = fontdue::Font::from_bytes(font_blob.data(), fontdue::FontSettings::default())
            .map_err(|err| crate::error::Error::Fontdue(err.to_string()))?;
        let name = name.or_else(|| font.name().map(|x| x.to_string())).ok_or(
            crate::error::Error::Other(format!("Unable to determine the name")),
        )?;
        let blob_id = BlobID(font_blob.id());
        self.fonts_cache
            .insert(name.clone(), FontInfo::new(font, font_blob));
        self.font_names.insert(blob_id, name.clone());
        return Ok(name);
    }

    pub fn push_font_file(
        &mut self,
        name: Option<String>,
        font_file: &Path,
    ) -> crate::error::Result<String> {
        let font_data = std::fs::read(&font_file).map_err(|err| crate::error::Error::IO(err))?;
        let font_blob = Blob::new(std::sync::Arc::new(font_data));
        let name = name.or_else(|| {
            font_file
                .file_stem()
                .map(|x| x.to_str())
                .flatten()
                .map(|x| x.to_string())
        });
        self.push_font(name, font_blob)
    }

    pub fn rasterized(
        &mut self,
        glyph_key: GlyphKey,
    ) -> Option<image::SubImage<&image::GrayImage>> {
        let info = self.glyph_store_info(glyph_key)?.clone();
        let atlas = self.atlas_cache.get(info.page_index)?;
        let data = atlas
            .data
            .view(info.rect.x, info.rect.y, info.rect.width, info.rect.height);
        Some(data)
    }

    pub fn glyph_index(&self, font_name: &str, character: char) -> Option<u32> {
        let font_info = self.fonts_cache.get(font_name)?;
        let index = font_info.font.lookup_glyph_index(character);
        if index == 0 {
            None
        } else {
            Some(index as u32)
        }
    }

    pub fn character(&self, font_name: &str, glyph_index: u32) -> Option<&char> {
        let font_info = self.fonts_cache.get(font_name)?;
        font_info.glyphs.get(&glyph_index)
    }

    pub fn metrics(&self, glyph_key: GlyphKey) -> Option<fontdue::Metrics> {
        let font_info = self.fonts_cache.get(&glyph_key.font_name)?;
        Some(
            font_info
                .font
                .metrics(glyph_key.character, glyph_key.size as f32),
        )
    }

    pub fn glyph_store_info(&mut self, glyph_key: GlyphKey) -> Option<&GlyphStoreInfo> {
        if self.glyph_cache.contains_key(&glyph_key) {
            return self.glyph_cache.get(&glyph_key);
        }

        let font_info = self.fonts_cache.get(&glyph_key.font_name)?;

        let (metrics, bitmap) = if font_info.font.has_glyph(glyph_key.character) {
            let index = font_info.font.lookup_glyph_index(glyph_key.character);
            let config = fontdue::layout::GlyphRasterConfig {
                glyph_index: index,
                px: glyph_key.size as f32,
                font_hash: font_info.font.file_hash(),
            };
            let (metrics, bitmap) = font_info.font.rasterize_config(config);
            (metrics, bitmap)
        } else {
            let (metrics, bitmap) = font_info
                .font
                .rasterize(glyph_key.character, glyph_key.size as f32);
            (metrics, bitmap)
        };

        let mut packed: Option<GlyphStoreInfo> = None;
        for (index, atlas) in self.atlas_cache.iter_mut().enumerate() {
            let rect = atlas
                .packer
                .insert(metrics.width as u32, metrics.height as u32);
            if let Some(rect) = rect {
                packed = Some(GlyphStoreInfo {
                    page_index: index,
                    rect,
                });
            }
        }

        if packed.is_none() {
            const LENGTH: u32 = atlas_length();
            let packer = rs_pack::skyline::SkylineBinPack::new(LENGTH, LENGTH);
            let data = image::GrayImage::new(LENGTH, LENGTH);
            let mut new_atlas = Atlas { packer, data };
            let rect = new_atlas
                .packer
                .insert(metrics.width as u32, metrics.height as u32);
            if let Some(rect) = rect {
                packed = Some(GlyphStoreInfo {
                    page_index: self.atlas_cache.len(),
                    rect,
                });
            }
            self.atlas_cache.push(new_atlas);
        }

        debug_assert_ne!(packed, None);
        let packed = packed?;
        let atlas = &mut self.atlas_cache[packed.page_index];
        let mut sub_data = atlas.data.sub_image(
            packed.rect.x as u32,
            packed.rect.y as u32,
            packed.rect.width as u32,
            packed.rect.height as u32,
        );
        for y in 0..packed.rect.height {
            for x in 0..packed.rect.width {
                let value = bitmap[(packed.rect.width * y + x) as usize];
                sub_data.put_pixel(x, y, image::Luma([value]));
            }
        }
        self.dirty_pages.insert(packed.page_index);
        let entry = self.glyph_cache.entry(glyph_key).or_insert_with(|| packed);
        Some(entry)
    }

    pub fn take_dirty_pages(&mut self) -> HashSet<usize> {
        let dirty_pages = self.dirty_pages.clone();
        self.dirty_pages.clear();
        return dirty_pages;
    }

    pub fn atlas_cache(&self) -> &[Atlas] {
        &self.atlas_cache
    }

    pub fn font(&self, name: &str) -> Option<&fontdue::Font> {
        self.fonts_cache.get(name).map(|x| &x.font)
    }

    pub fn font_data(&self, name: &str) -> Option<Blob<u8>> {
        self.fonts_cache.get(name).map(|x| x.data.clone())
    }

    pub fn font_name<T: Into<BlobID>>(&self, blob_id: T) -> Option<&String> {
        self.font_names.get(&blob_id.into())
    }
}

pub const fn atlas_length() -> u32 {
    2048
}

#[cfg(test)]
pub mod test {
    use crate::cache::{FontCache, GlyphKey};
    use fontdue::layout::LayoutSettings;
    use image::GenericImage;

    #[test]
    fn test() {
        let mut font_cache = FontCache::new(1024, 2, 2);
        let font_file = crate::test::test_font_file_path();
        let font_name = font_cache.push_font_file(None, &font_file).unwrap();
        println!("{}", font_name);

        let glyph_key = GlyphKey {
            font_name,
            character: 'A',
            size: 150,
        };
        let _ = font_cache.glyph_store_info(glyph_key.clone()).unwrap();
        let rasterized = font_cache.rasterized(glyph_key).unwrap();
        let image = rasterized.to_image();
        let output = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("test.png");
        println!("Save to {:?}", &output);
        image.save(output).unwrap();
    }

    #[test]
    fn layout_test() {
        let mut font_cache = FontCache::new(1024, 2, 2);
        let font_file = crate::test::test_font_file_path();
        let font_name = font_cache.push_font_file(None, &font_file).unwrap();
        let font = font_cache.font(&font_name).unwrap();
        let fonts = &[font];
        let mut layout =
            fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            ..LayoutSettings::default()
        });
        layout.append(fonts, &fontdue::layout::TextStyle::new("Hello \n", 25.0, 0));
        layout.append(fonts, &fontdue::layout::TextStyle::new(" world!", 80.0, 0));
        let mut gray_image = image::GrayImage::new(512, 256);

        for glyph in layout.glyphs() {
            if glyph.char_data.is_whitespace() {
                continue;
            }
            let mut sub_image = gray_image.sub_image(
                glyph.x as u32,
                glyph.y as u32,
                glyph.width as u32,
                glyph.height as u32,
            );
            let rasterized = font_cache.rasterized(GlyphKey {
                font_name: font_name.clone(),
                character: glyph.parent,
                size: glyph.key.px as u32,
            });
            if let Some(rasterized) = rasterized {
                let other = rasterized.to_image();
                let _ = sub_image.copy_from(&other, 0, 0);
            }
        }

        let output = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("layout_test.png");
        println!("Save to {:?}", &output);
        gray_image.save(output).unwrap();
    }
}
