//! The game's textures on the GPU: the layered arrays by size, their levels, and the uploads.

use super::*;

impl Gpu {
    /// A new array of `capacity` layers of `size`, bound on the active unit.
    pub(super) fn make_array(
        &self,
        (width, height): (usize, usize),
        capacity: i32,
    ) -> TextureArray {
        let gl = &self.gl;
        let name = gl.gen_texture();
        gl.bind_texture(TEXTURE_2D_ARRAY, name);
        gl.tex_image_3d_empty(0x8058, width as i32, height as i32, capacity);
        // One level, which the sampler objects' filters read alone, until a batch asks for more.
        gl.tex_parameter(TEXTURE_2D_ARRAY, 0x813D, 0);
        let mut array = TextureArray {
            name,
            capacity,
            next: 0,
            free: Vec::new(),
            levelled: false,
        };
        if self.mip_levels {
            self.allocate_levels(&mut array, (width, height));
        }
        array
    }

    /// The smaller levels of the bound array of `size` made room for (the layers' own are
    /// uploaded with each texture, or made at once by `level_arrays`).
    pub(super) fn allocate_levels(
        &self,
        array: &mut TextureArray,
        (width, height): (usize, usize),
    ) {
        let gl = &self.gl;
        let count = mip_levels::level_count(width, height);
        for level in 1..count {
            gl.tex_image_3d_level_empty(
                level as i32,
                0x8058,
                mip_levels::side_at(width, level) as i32,
                mip_levels::side_at(height, level) as i32,
                array.capacity,
            );
        }
        gl.tex_parameter(TEXTURE_2D_ARRAY, 0x813D, count as i32 - 1);
        array.levelled = true;
    }

    /// Every array given its smaller levels, made from the layers it holds: the first time a
    /// batch asks for mipmapping.
    pub(super) fn level_arrays(&mut self) {
        self.mip_levels = true;
        let sizes: Vec<_> = self.arrays.keys().copied().collect();
        for size in sizes {
            let Some(mut array) = self.arrays.remove(&size) else {
                continue;
            };
            self.gl.bind_texture(TEXTURE_2D_ARRAY, array.name);
            if !array.levelled {
                self.allocate_levels(&mut array, size);
                self.gl.generate_mipmap_array();
            }
            self.arrays.insert(size, array);
        }
    }

    /// A layer of the array of `size` for a new texture: one given back, a new one, or (the
    /// array full) one of a new array twice its size, the old one's layers copied into it.
    pub(super) fn new_layer(&mut self, size: (usize, usize)) -> i32 {
        if !self.arrays.contains_key(&size) {
            let array = self.make_array(size, FIRST_LAYERS);
            self.arrays.insert(size, array);
        }
        let full = self
            .arrays
            .get(&size)
            .is_some_and(|a| a.free.is_empty() && a.next == a.capacity);
        if full {
            self.grow(size);
        }
        let Some(array) = self.arrays.get_mut(&size) else {
            unreachable!("made above");
        };
        array.free.pop().unwrap_or_else(|| {
            array.next += 1;
            array.next - 1
        })
    }

    /// The array of `size` twice as big, its layers kept.
    pub(super) fn grow(&mut self, size: (usize, usize)) {
        let Some(old) = self.arrays.remove(&size) else {
            return;
        };
        let mut bigger = self.make_array(size, old.capacity * 2);
        let gl = &self.gl;
        gl.bind_framebuffer(READ_FRAMEBUFFER, self.copy_framebuffer);
        for layer in 0..old.next {
            gl.framebuffer_texture_layer(READ_FRAMEBUFFER, old.name, layer);
            gl.copy_into_layer(layer, size.0 as i32, size.1 as i32);
        }
        gl.bind_framebuffer(READ_FRAMEBUFFER, 0);
        if bigger.levelled {
            // Only level 0 was copied.
            gl.bind_texture(TEXTURE_2D_ARRAY, bigger.name);
            gl.generate_mipmap_array();
        }
        gl.delete_texture(old.name);
        (bigger.next, bigger.free) = (old.next, old.free);
        self.arrays.insert(size, bigger);
    }

    /// The two arrays a pass whose textures are of `size` reads, onto units `unit` and
    /// `unit + 1` (the active unit left at `unit + 1`).
    pub(super) fn bind_arrays(&self, unit: u32, size: (usize, usize)) {
        let name = |size| self.arrays.get(&size).map_or(0, |a| a.name);
        let (a, b) = if MAIN_SIZES.contains(&size) {
            (name(MAIN_SIZES[0]), name(MAIN_SIZES[1]))
        } else {
            (name(size), 0)
        };
        let gl = &self.gl;
        gl.active_texture(TEXTURE0 + unit);
        gl.bind_texture(TEXTURE_2D_ARRAY, a);
        gl.active_texture(TEXTURE0 + unit + 1);
        gl.bind_texture(TEXTURE_2D_ARRAY, b);
    }

    pub(super) fn bring_texture_up(&mut self, note: &TextureNote, bytes: &[u8]) {
        crate::render::UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        crate::render::UPLOAD_BYTES
            .fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
        let size = (note.width, note.height);
        let known = self
            .textures
            .get(&note.pixels)
            .map(|t| ((t.width, t.height), t.layer));
        let layer = match known {
            Some((was, layer)) if was == size => layer,
            Some((was, layer)) => {
                self.give_back(was, layer);
                self.new_layer(size)
            }
            None => self.new_layer(size),
        };
        let gl = &self.gl;
        gl.bind_texture(
            TEXTURE_2D_ARRAY,
            self.arrays.get(&size).map_or(0, |a| a.name),
        );
        // `bytes` holds the whole picture (`texture_as_bgra`).
        gl.tex_sub_image_layer(layer, note.width as i32, note.height as i32, bytes);
        if self.arrays.get(&size).is_some_and(|a| a.levelled) {
            let whole = note.width * note.height * 4;
            for (n, (w, h, level)) in
                mip_levels::lower_levels(&bytes[..whole.min(bytes.len())], note.width, note.height)
                    .iter()
                    .enumerate()
            {
                gl.tex_sub_image_layer_level(n as i32 + 1, layer, *w as i32, *h as i32, level);
            }
        }
        self.textures.insert(
            note.pixels,
            CachedTexture {
                layer,
                width: note.width,
                height: note.height,
            },
        );
    }

    /// The textures whose memory is gone let go of.
    pub(super) fn forget(&mut self, gone: &[crate::render::TextureId]) {
        for pixels in gone {
            if let Some(texture) = self.textures.remove(pixels) {
                self.give_back((texture.width, texture.height), texture.layer);
            }
        }
    }

    /// A layer of the array of `size` free again.
    pub(super) fn give_back(&mut self, size: (usize, usize), layer: i32) {
        if let Some(array) = self.arrays.get_mut(&size) {
            array.free.push(layer);
        }
    }
}
