#![allow(unsafe_code)]
//! SDL3's GPU API (`SDL_GPU*`; Vulkan on the PC, Metal on macOS), the only place that calls it. The objects are
//! small `Copy` handles (a raw pointer kept private, `NONE` for "not made yet"); making and
//! releasing them is explicit, on `Device`, and the caller releases each once and stops using
//! it, as it would the SDL calls. Nothing here keeps a count or checks a handle against the
//! device: what the wrappers do check is what Rust memory is read or written, which is that a
//! slice is at least as long as the call needs and that a copy stays inside the transfer buffer
//! or the buffer it names (their sizes are kept in the handles). Errors are SDL's own text.
//!
//! The handles are used under `Gpu`'s lock, one thread at a time, as SDL asks: `unsafe impl
//! Send` on each, for that reason. The wrappers on the per-draw path are `#[inline]`.

use sdl3_sys::everything::*;
use std::ffi::CStr;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the device made can clamp depth (see `Device::create`).
static DEPTH_CLAMP: AtomicBool = AtomicBool::new(true);

/// Whether the device made clamps depth; a pipeline that wants the clamp clips when not.
pub fn depth_clamp() -> bool {
    DEPTH_CLAMP.load(Ordering::Relaxed)
}

/// SDL's last error text, copied.
pub fn error() -> String {
    // SAFETY: SDL's own text, valid until its next call, copied at once.
    unsafe { std::ffi::CStr::from_ptr(SDL_GetError()) }
        .to_string_lossy()
        .into_owned()
}

/// SDL's video subsystem started (the test's: the application starts it elsewhere).
#[cfg(test)]
pub fn init_video() {
    // SAFETY: no pointers; SDL's start-up.
    unsafe { SDL_Init(SDL_INIT_VIDEO) };
}

macro_rules! handle {
    ($(#[$meta:meta])* $name:ident($raw:ty)) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name($raw);
        // SAFETY: SDL's object, used one thread at a time (see the module header).
        unsafe impl Send for $name {}
        impl $name {
            /// Not made.
            pub const NONE: $name = $name(null_mut());
            /// Whether it is not made.
            pub fn is_none(self) -> bool {
                self.0.is_null()
            }
        }
    };
}
/// A GPU device.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Device(*mut SDL_GPUDevice);
// SAFETY: SDL's object, used one thread at a time (see the module header).
unsafe impl Send for Device {}
impl Device {
    /// The SDL object, for the platform's own SDL calls.
    pub fn as_raw(self) -> *mut SDL_GPUDevice {
        self.0
    }
}
impl Texture {
    /// The SDL object, for the platform's own SDL calls.
    pub fn as_raw(self) -> *mut SDL_GPUTexture {
        self.0
    }
}
handle!(Shader(*mut SDL_GPUShader));
handle!(Sampler(*mut SDL_GPUSampler));
handle!(Texture(*mut SDL_GPUTexture));
handle!(Pipeline(*mut SDL_GPUGraphicsPipeline));

macro_rules! sized_handle {
    ($(#[$meta:meta])* $name:ident($raw:ty)) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name($raw, u32);
        // SAFETY: SDL's object, used one thread at a time (see the module header).
        unsafe impl Send for $name {}
        impl $name {
            /// Not made.
            pub const NONE: $name = $name(null_mut(), 0);
            pub fn is_none(self) -> bool {
                self.0.is_null()
            }
        }
    };
}
sized_handle!(
    /// A buffer on the GPU, with its size in bytes.
    Buffer(*mut SDL_GPUBuffer)
);
sized_handle!(
    /// A buffer the CPU maps, with its size in bytes.
    Transfer(*mut SDL_GPUTransferBuffer)
);

impl Transfer {
    /// The size it was made with, in bytes.
    pub fn size(self) -> u32 {
        self.1
    }
}

/// A submitted command buffer's fence: it is signalled when the GPU has done the buffer.
#[derive(Debug)]
pub struct Fence(*mut SDL_GPUFence);
// SAFETY: SDL's object, used one thread at a time (see the module header).
unsafe impl Send for Fence {}

/// A command buffer, until it is submitted.
pub struct CommandBuffer(*mut SDL_GPUCommandBuffer);
/// A copy pass, until it is ended.
pub struct CopyPass(*mut SDL_GPUCopyPass);
/// A render pass, until it is ended.
pub struct RenderPass(*mut SDL_GPURenderPass);

/// What a graphics pipeline is made of besides its shaders (the pipelines have one colour
/// target and triangle lists).
pub struct PipelineSpec<'a> {
    /// The one vertex buffer's pitch and the attributes read from it; none: no vertices.
    pub vertices: Option<(u32, &'a [SDL_GPUVertexAttribute])>,
    pub colour: SDL_GPUColorTargetDescription,
    pub rasterizer: SDL_GPURasterizerState,
    /// The depth test's state and the depth target's format; none: no depth target.
    pub depth: Option<(SDL_GPUDepthStencilState, SDL_GPUTextureFormat)>,
}

/// A shader's code in each format a device may take: SPIR-V (Vulkan), and on Apple systems also
/// Metal's source (MSL, made from the SPIR-V by `tools/compile_shaders.sh`).
#[derive(Clone, Copy)]
pub struct ShaderCode {
    pub spirv: &'static [u8],
    #[cfg(apple)]
    pub msl: &'static [u8],
}

impl Device {
    /// A device that takes our shaders (SPIR-V; on macOS SPIR-V or MSL); none (SDL's error is
    /// `error()`) when the system has no such.
    ///
    /// SDL's Vulkan driver refuses a device that lacks a feature it was not told to do without, and
    /// by default asks for three the renderer never uses (shader clip distance, anisotropic
    /// filtering, indirect draws with a first instance), which some phones' drivers (Mali) do not
    /// offer. They are switched off here on every system: nothing drawn changes. Depth clamping
    /// the scene's pipeline does use, so a device is tried with it first and, when the system has
    /// none that takes it, once more without (`depth_clamp()` then says so, and the pipelines clip
    /// instead of clamping).
    pub fn create() -> Option<Device> {
        for clamp in [true, false] {
            if let Some(device) = Self::create_with(clamp) {
                DEPTH_CLAMP.store(clamp, Ordering::Relaxed);
                return Some(device);
            }
        }
        None
    }

    fn create_with(depth_clamp: bool) -> Option<Device> {
        // SAFETY: the properties are made, filled with names that are NUL-ended constants and
        // plain values, used by one call and destroyed after it.
        let device = unsafe {
            let props = SDL_CreateProperties();
            // SDL's properties default to its debug mode (validation, which on Metal stops the
            // program at its first complaint); the plain device, as `SDL_CreateGPUDevice`'s
            // `debug_mode` false gave.
            SDL_SetBooleanProperty(props, SDL_PROP_GPU_DEVICE_CREATE_DEBUGMODE_BOOLEAN, false);
            SDL_SetBooleanProperty(
                props,
                SDL_PROP_GPU_DEVICE_CREATE_SHADERS_SPIRV_BOOLEAN,
                true,
            );
            #[cfg(apple)]
            SDL_SetBooleanProperty(props, SDL_PROP_GPU_DEVICE_CREATE_SHADERS_MSL_BOOLEAN, true);
            SDL_SetBooleanProperty(
                props,
                SDL_PROP_GPU_DEVICE_CREATE_FEATURE_CLIP_DISTANCE_BOOLEAN,
                false,
            );
            SDL_SetBooleanProperty(
                props,
                SDL_PROP_GPU_DEVICE_CREATE_FEATURE_ANISOTROPY_BOOLEAN,
                false,
            );
            SDL_SetBooleanProperty(
                props,
                SDL_PROP_GPU_DEVICE_CREATE_FEATURE_INDIRECT_DRAW_FIRST_INSTANCE_BOOLEAN,
                false,
            );
            SDL_SetBooleanProperty(
                props,
                SDL_PROP_GPU_DEVICE_CREATE_FEATURE_DEPTH_CLAMPING_BOOLEAN,
                depth_clamp,
            );
            let device = SDL_CreateGPUDeviceWithProperties(props);
            SDL_DestroyProperties(props);
            device
        };
        (!device.is_null()).then_some(Device(device))
    }

    /// The device destroyed. Only for one nothing else holds: no renderer, texture, buffer or
    /// pipeline made on it is alive.
    pub fn destroy(self) {
        // SAFETY: a device `create` made, which the caller says nothing uses; it is not used
        // again (`Device` is `Copy`, and the caller drops its copies).
        unsafe { SDL_DestroyGPUDevice(self.0) };
    }

    /// The format the device's shaders are made from, of the ones asked for (SPIR-V where it
    /// takes it), its code in `code`, and the code's entry point (spirv-cross names MSL's
    /// `main0`).
    fn shader_format(
        self,
        code: ShaderCode,
    ) -> (SDL_GPUShaderFormat, &'static [u8], &'static CStr) {
        #[cfg(apple)]
        {
            // SAFETY: no pointers.
            let formats = unsafe { SDL_GetGPUShaderFormats(self.0) };
            if formats & SDL_GPU_SHADERFORMAT_SPIRV == 0 {
                return (SDL_GPU_SHADERFORMAT_MSL, code.msl, c"main0");
            }
        }
        (SDL_GPU_SHADERFORMAT_SPIRV, code.spirv, c"main")
    }

    /// The name of the driver the device runs on.
    pub fn driver(self) -> String {
        // SAFETY: the device's own constant text, copied at once.
        unsafe { std::ffi::CStr::from_ptr(SDL_GetGPUDeviceDriver(self.0)) }
            .to_string_lossy()
            .into_owned()
    }

    /// A shader from its code in the device's format.
    pub fn create_shader(
        self,
        code: ShaderCode,
        stage: SDL_GPUShaderStage,
        samplers: u32,
        uniforms: u32,
    ) -> Result<Shader, String> {
        let (format, code, entry) = self.shader_format(code);
        let info = SDL_GPUShaderCreateInfo {
            code_size: code.len(),
            code: code.as_ptr(),
            entrypoint: entry.as_ptr(),
            format,
            stage,
            num_samplers: samplers,
            num_uniform_buffers: uniforms,
            ..Default::default()
        };
        // SAFETY: `info` and the code it points to live through the call.
        let shader = unsafe { SDL_CreateGPUShader(self.0, &raw const info) };
        (!shader.is_null())
            .then_some(Shader(shader))
            .ok_or_else(error)
    }

    pub fn create_sampler(self, info: &SDL_GPUSamplerCreateInfo) -> Result<Sampler, String> {
        // SAFETY: `info` is plain values (no pointers) and lives through the call.
        let sampler = unsafe { SDL_CreateGPUSampler(self.0, info) };
        (!sampler.is_null())
            .then_some(Sampler(sampler))
            .ok_or_else(error)
    }

    pub fn create_texture(
        self,
        format: SDL_GPUTextureFormat,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
    ) -> Result<Texture, String> {
        self.create_texture_levels(format, width, height, usage, 1)
    }

    /// As `create_texture`, with `levels` mipmap levels (the full size counts as one).
    pub fn create_texture_levels(
        self,
        format: SDL_GPUTextureFormat,
        width: u32,
        height: u32,
        usage: SDL_GPUTextureUsageFlags,
        levels: u32,
    ) -> Result<Texture, String> {
        let info = SDL_GPUTextureCreateInfo {
            r#type: SDL_GPU_TEXTURETYPE_2D,
            format,
            usage,
            width,
            height,
            layer_count_or_depth: 1,
            num_levels: levels,
            sample_count: SDL_GPU_SAMPLECOUNT_1,
            props: SDL_PropertiesID(0),
        };
        // SAFETY: `info` lives through the call.
        let texture = unsafe { SDL_CreateGPUTexture(self.0, &raw const info) };
        (!texture.is_null())
            .then_some(Texture(texture))
            .ok_or_else(error)
    }

    /// A graphics pipeline of triangle lists from the two shaders.
    pub fn create_pipeline(
        self,
        vertex_shader: Shader,
        fragment_shader: Shader,
        spec: &PipelineSpec,
    ) -> Result<Pipeline, String> {
        let buffer = spec.vertices.map(|(pitch, _)| {
            [SDL_GPUVertexBufferDescription {
                slot: 0,
                pitch,
                input_rate: SDL_GPU_VERTEXINPUTRATE_VERTEX,
                instance_step_rate: 0,
            }]
        });
        let attributes = spec.vertices.map_or(&[][..], |(_, attributes)| attributes);
        let colour = [spec.colour];
        let mut info = SDL_GPUGraphicsPipelineCreateInfo {
            vertex_shader: vertex_shader.0,
            fragment_shader: fragment_shader.0,
            primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            rasterizer_state: spec.rasterizer,
            target_info: SDL_GPUGraphicsPipelineTargetInfo {
                color_target_descriptions: colour.as_ptr(),
                num_color_targets: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        if let Some(buffer) = &buffer {
            info.vertex_input_state = SDL_GPUVertexInputState {
                vertex_buffer_descriptions: buffer.as_ptr(),
                num_vertex_buffers: 1,
                vertex_attributes: attributes.as_ptr(),
                num_vertex_attributes: attributes.len() as u32,
            };
        }
        if let Some((state, format)) = spec.depth {
            info.depth_stencil_state = state;
            info.target_info.depth_stencil_format = format;
            info.target_info.has_depth_stencil_target = true;
        }
        // SAFETY: `info` and the arrays it points to (`colour`, `buffer`, the attributes) live
        // through the call.
        let pipeline = unsafe { SDL_CreateGPUGraphicsPipeline(self.0, &raw const info) };
        (!pipeline.is_null())
            .then_some(Pipeline(pipeline))
            .ok_or_else(error)
    }

    pub fn create_transfer(
        self,
        usage: SDL_GPUTransferBufferUsage,
        size: u32,
    ) -> Result<Transfer, String> {
        let info = SDL_GPUTransferBufferCreateInfo {
            usage,
            size,
            props: SDL_PropertiesID(0),
        };
        // SAFETY: `info` lives through the call.
        let transfer = unsafe { SDL_CreateGPUTransferBuffer(self.0, &raw const info) };
        (!transfer.is_null())
            .then_some(Transfer(transfer, size))
            .ok_or_else(error)
    }

    /// A buffer of `size` bytes for vertices.
    pub fn create_vertex_buffer(self, size: u32) -> Result<Buffer, String> {
        let info = SDL_GPUBufferCreateInfo {
            usage: SDL_GPU_BUFFERUSAGE_VERTEX,
            size,
            props: SDL_PropertiesID(0),
        };
        // SAFETY: `info` lives through the call.
        let buffer = unsafe { SDL_CreateGPUBuffer(self.0, &raw const info) };
        (!buffer.is_null())
            .then_some(Buffer(buffer, size))
            .ok_or_else(error)
    }

    /// Lets go of a texture (not `NONE`); SDL waits for the work that uses it.
    pub fn release_texture(self, texture: Texture) {
        // SAFETY: the caller's, made by this device and released once.
        unsafe { SDL_ReleaseGPUTexture(self.0, texture.0) };
    }

    /// Lets go of a shader (a pipeline made from it keeps its own copy).
    pub fn release_shader(self, shader: Shader) {
        // SAFETY: as `release_texture`.
        unsafe { SDL_ReleaseGPUShader(self.0, shader.0) };
    }

    pub fn release_sampler(self, sampler: Sampler) {
        // SAFETY: as `release_texture`.
        unsafe { SDL_ReleaseGPUSampler(self.0, sampler.0) };
    }

    pub fn release_pipeline(self, pipeline: Pipeline) {
        // SAFETY: as `release_texture`.
        unsafe { SDL_ReleaseGPUGraphicsPipeline(self.0, pipeline.0) };
    }

    pub fn release_transfer(self, transfer: Transfer) {
        // SAFETY: as `release_texture`.
        unsafe { SDL_ReleaseGPUTransferBuffer(self.0, transfer.0) };
    }

    pub fn release_buffer(self, buffer: Buffer) {
        // SAFETY: as `release_texture`.
        unsafe { SDL_ReleaseGPUBuffer(self.0, buffer.0) };
    }

    /// `bytes` into the transfer buffer at `at`; `cycle` for a buffer the GPU may still be
    /// reading.
    pub fn fill_transfer(
        self,
        transfer: Transfer,
        at: usize,
        bytes: &[u8],
        cycle: bool,
    ) -> Result<(), String> {
        assert!(
            at + bytes.len() <= transfer.1 as usize,
            "past the transfer buffer"
        );
        // SAFETY: the buffer holds `at + bytes.len()` bytes (checked above), mapped only here
        // and unmapped before the end.
        unsafe {
            let map = SDL_MapGPUTransferBuffer(self.0, transfer.0, cycle);
            if map.is_null() {
                return Err(error());
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), map.cast::<u8>().add(at), bytes.len());
            SDL_UnmapGPUTransferBuffer(self.0, transfer.0);
        }
        Ok(())
    }

    /// Each of `parts` (a byte offset and its bytes) into the transfer buffer, in one mapping,
    /// which cycles the buffer: the bytes between the parts are undefined afterwards.
    pub fn fill_transfer_parts(
        self,
        transfer: Transfer,
        parts: &[(usize, &[u8])],
    ) -> Result<(), String> {
        assert!(
            parts
                .iter()
                .all(|(at, bytes)| at + bytes.len() <= transfer.1 as usize),
            "past the transfer buffer"
        );
        // SAFETY: every part lies within the buffer (checked above); the mapping is made only
        // here and unmapped before the end.
        unsafe {
            let map = SDL_MapGPUTransferBuffer(self.0, transfer.0, true);
            if map.is_null() {
                return Err(error());
            }
            for (at, bytes) in parts {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    map.cast::<u8>().add(*at),
                    bytes.len(),
                );
            }
            SDL_UnmapGPUTransferBuffer(self.0, transfer.0);
        }
        Ok(())
    }

    /// The first `out.len()` bytes of a transfer buffer the GPU has written (the caller has
    /// waited for it) into `out`.
    pub fn read_transfer(self, transfer: Transfer, out: &mut [u8]) -> Result<(), String> {
        assert!(out.len() <= transfer.1 as usize, "past the transfer buffer");
        // SAFETY: the buffer holds `out.len()` bytes (checked above), mapped only here and
        // unmapped before the end.
        unsafe {
            let map = SDL_MapGPUTransferBuffer(self.0, transfer.0, false);
            if map.is_null() {
                return Err(error());
            }
            std::ptr::copy_nonoverlapping(map.cast::<u8>(), out.as_mut_ptr(), out.len());
            SDL_UnmapGPUTransferBuffer(self.0, transfer.0);
        }
        Ok(())
    }

    /// Waits until the GPU has done what `fence` is for, then releases the fence.
    pub fn wait_fence(self, fence: Fence) {
        // SAFETY: the fence is ours (made by `submit_with_fence`) and is given up here.
        unsafe {
            SDL_WaitForGPUFences(self.0, true, &raw const fence.0, 1);
            SDL_ReleaseGPUFence(self.0, fence.0);
        }
    }

    pub fn acquire_command_buffer(self) -> Result<CommandBuffer, String> {
        // SAFETY: the device is alive; the buffer is the caller's until submitted.
        let buffer = unsafe { SDL_AcquireGPUCommandBuffer(self.0) };
        (!buffer.is_null())
            .then_some(CommandBuffer(buffer))
            .ok_or_else(error)
    }
}

impl CommandBuffer {
    pub fn begin_copy_pass(&self) -> CopyPass {
        // SAFETY: our open command buffer.
        CopyPass(unsafe { SDL_BeginGPUCopyPass(self.0) })
    }

    /// A render pass over `colour` (its contents kept) and `depth` (cleared to 1.0 when
    /// `clear_depth`, else kept; stored).
    pub fn begin_render_pass(
        &self,
        colour: Texture,
        depth: Texture,
        clear_depth: bool,
    ) -> RenderPass {
        let colour = SDL_GPUColorTargetInfo {
            texture: colour.0,
            load_op: SDL_GPU_LOADOP_LOAD,
            store_op: SDL_GPU_STOREOP_STORE,
            ..Default::default()
        };
        let depth = SDL_GPUDepthStencilTargetInfo {
            texture: depth.0,
            clear_depth: 1.0,
            load_op: if clear_depth {
                SDL_GPU_LOADOP_CLEAR
            } else {
                SDL_GPU_LOADOP_LOAD
            },
            store_op: SDL_GPU_STOREOP_STORE,
            stencil_load_op: SDL_GPU_LOADOP_DONT_CARE,
            stencil_store_op: SDL_GPU_STOREOP_DONT_CARE,
            ..Default::default()
        };
        // SAFETY: our open command buffer; the two records live through the call.
        RenderPass(unsafe {
            SDL_BeginGPURenderPass(self.0, &raw const colour, 1, &raw const depth)
        })
    }

    /// A render pass into `colour` alone, without a depth target; the target's old pixels are
    /// not kept (every pixel is written).
    pub fn begin_plain_pass(&self, colour: Texture) -> RenderPass {
        let colour = SDL_GPUColorTargetInfo {
            texture: colour.0,
            load_op: SDL_GPU_LOADOP_DONT_CARE,
            store_op: SDL_GPU_STOREOP_STORE,
            ..Default::default()
        };
        // SAFETY: our open command buffer; the record lives through the call; no depth target
        // is a null pointer.
        RenderPass(unsafe {
            SDL_BeginGPURenderPass(self.0, &raw const colour, 1, std::ptr::null())
        })
    }

    /// The top-left `from` (width, height) pixels of `source` copied to the top-left `to` pixels
    /// of `destination`, each destination pixel the nearest source pixel, or with `smooth` the
    /// bilinear mean of the four around it (outside a pass). Both
    /// textures are ours, made for blitting (a sampler and a colour target), and the regions are
    /// inside them.
    pub fn blit(
        &self,
        source: Texture,
        from: (u32, u32),
        destination: Texture,
        to: (u32, u32),
        smooth: bool,
    ) {
        let region = |texture: Texture, (w, h): (u32, u32)| SDL_GPUBlitRegion {
            texture: texture.0,
            w,
            h,
            ..Default::default()
        };
        let info = SDL_GPUBlitInfo {
            source: region(source, from),
            destination: region(destination, to),
            load_op: SDL_GPU_LOADOP_DONT_CARE,
            filter: if smooth {
                SDL_GPU_FILTER_LINEAR
            } else {
                SDL_GPU_FILTER_NEAREST
            },
            ..Default::default()
        };
        // SAFETY: our open command buffer, no pass open; `info` lives through the call.
        unsafe { SDL_BlitGPUTexture(self.0, &raw const info) };
    }

    /// Two floats for the vertex shader's uniform buffer `slot`.
    pub fn push_vertex_uniform(&self, slot: u32, values: &[f32; 2]) {
        // SAFETY: eight bytes, copied by the call.
        unsafe { SDL_PushGPUVertexUniformData(self.0, slot, values.as_ptr().cast(), 8) };
    }

    /// Eight dwords for the fragment shader's uniform buffer `slot`.
    #[inline]
    pub fn push_fragment_uniform(&self, slot: u32, values: &[u32; 8]) {
        // SAFETY: 32 bytes, copied by the call.
        unsafe { SDL_PushGPUFragmentUniformData(self.0, slot, values.as_ptr().cast(), 32) };
    }

    /// Submitted; the picture is the GPU's from here.
    pub fn submit(self) -> Result<(), String> {
        // SAFETY: our command buffer, given up here.
        if unsafe { SDL_SubmitGPUCommandBuffer(self.0) } {
            Ok(())
        } else {
            Err(error())
        }
    }

    /// Submitted, with a fence to wait for later (`Device::wait_fence`).
    pub fn submit_with_fence(self) -> Result<Fence, String> {
        // SAFETY: our command buffer, given up here; the fence is handed to the caller, who
        // gives it back to `Device::wait_fence`.
        let fence = unsafe { SDL_SubmitGPUCommandBufferAndAcquireFence(self.0) };
        if fence.is_null() {
            Err(error())
        } else {
            Ok(Fence(fence))
        }
    }

    /// Submitted, and waited for until the GPU has done it.
    pub fn submit_and_wait(self, device: Device) -> Result<(), String> {
        // SAFETY: our command buffer, given up here; the fence is ours, waited for and released.
        unsafe {
            let fence = SDL_SubmitGPUCommandBufferAndAcquireFence(self.0);
            if fence.is_null() {
                return Err(error());
            }
            SDL_WaitForGPUFences(device.0, true, &raw const fence, 1);
            SDL_ReleaseGPUFence(device.0, fence);
        }
        Ok(())
    }
}

impl CopyPass {
    /// `size` bytes of the transfer buffer's start into the start of `buffer`.
    pub fn upload_buffer(&self, transfer: Transfer, buffer: Buffer, size: u32) {
        assert!(size <= transfer.1 && size <= buffer.1, "past the buffers");
        let from = SDL_GPUTransferBufferLocation {
            transfer_buffer: transfer.0,
            offset: 0,
        };
        let to = SDL_GPUBufferRegion {
            buffer: buffer.0,
            offset: 0,
            size,
        };
        // SAFETY: our open pass; the records live through the call; the sizes are within both
        // buffers (checked above).
        unsafe { SDL_UploadToGPUBuffer(self.0, &raw const from, &raw const to, true) };
    }

    /// A whole picture of `width` x `height` four-byte pixels from the transfer buffer at
    /// `offset` into `texture`.
    pub fn upload_texture(
        &self,
        transfer: Transfer,
        offset: u32,
        texture: Texture,
        width: u32,
        height: u32,
    ) {
        self.upload_texture_level(transfer, offset, texture, 0, width, height);
    }

    /// As `upload_texture`, into mipmap level `level` (`width` x `height` its size).
    pub fn upload_texture_level(
        &self,
        transfer: Transfer,
        offset: u32,
        texture: Texture,
        level: u32,
        width: u32,
        height: u32,
    ) {
        assert!(
            u64::from(offset) + u64::from(width) * u64::from(height) * 4 <= u64::from(transfer.1),
            "past the transfer buffer"
        );
        let from = SDL_GPUTextureTransferInfo {
            transfer_buffer: transfer.0,
            offset,
            pixels_per_row: width,
            rows_per_layer: height,
        };
        let to = SDL_GPUTextureRegion {
            texture: texture.0,
            mip_level: level,
            w: width,
            h: height,
            d: 1,
            ..Default::default()
        };
        // The first level may take a fresh backing (the GPU may still be reading the old one); a
        // later level must not, or it would discard the levels already sent.
        // SAFETY: our open pass; the records live through the call; the picture is within the
        // transfer buffer (checked above).
        unsafe { SDL_UploadToGPUTexture(self.0, &raw const from, &raw const to, level == 0) };
    }

    /// `rows` rows of `width` four-byte pixels from the transfer buffer at `offset` into
    /// `texture`, from its row `first`; the texture's other rows are kept (it is not cycled).
    pub fn upload_texture_rows(
        &self,
        transfer: Transfer,
        offset: u32,
        texture: Texture,
        first: u32,
        width: u32,
        rows: u32,
    ) {
        assert!(
            u64::from(offset) + u64::from(width) * u64::from(rows) * 4 <= u64::from(transfer.1),
            "past the transfer buffer"
        );
        let from = SDL_GPUTextureTransferInfo {
            transfer_buffer: transfer.0,
            offset,
            pixels_per_row: width,
            rows_per_layer: rows,
        };
        let to = SDL_GPUTextureRegion {
            texture: texture.0,
            y: first,
            w: width,
            h: rows,
            d: 1,
            ..Default::default()
        };
        // SAFETY: our open pass; the records live through the call; the rows are within the
        // transfer buffer (checked above).
        unsafe { SDL_UploadToGPUTexture(self.0, &raw const from, &raw const to, false) };
    }

    /// The `width` x `height` four-byte pixels from the transfer buffer at `offset` into
    /// `texture` at (`x`, `y`); the texture's other pixels are kept (it is not cycled).
    pub fn upload_texture_region(
        &self,
        transfer: Transfer,
        offset: u32,
        texture: Texture,
        [x, y, width, height]: [u32; 4],
    ) {
        assert!(
            u64::from(offset) + u64::from(width) * u64::from(height) * 4 <= u64::from(transfer.1),
            "past the transfer buffer"
        );
        let from = SDL_GPUTextureTransferInfo {
            transfer_buffer: transfer.0,
            offset,
            pixels_per_row: width,
            rows_per_layer: height,
        };
        let to = SDL_GPUTextureRegion {
            texture: texture.0,
            x,
            y,
            w: width,
            h: height,
            d: 1,
            ..Default::default()
        };
        // SAFETY: our open pass; the records live through the call; the pixels are within the
        // transfer buffer (checked above).
        unsafe { SDL_UploadToGPUTexture(self.0, &raw const from, &raw const to, false) };
    }

    /// `texture`'s `width` x `height` pixels into the start of the transfer buffer.
    pub fn download_texture(&self, texture: Texture, width: u32, height: u32, transfer: Transfer) {
        self.download_texture_at(texture, (width, height), 4, transfer, 0);
    }

    /// As `download_texture` for a texture of `bytes_per_pixel` bytes a pixel, its picture
    /// written `offset` bytes into the transfer buffer.
    pub fn download_texture_at(
        &self,
        texture: Texture,
        (width, height): (u32, u32),
        bytes_per_pixel: u32,
        transfer: Transfer,
        offset: u32,
    ) {
        assert!(
            u64::from(width) * u64::from(height) * u64::from(bytes_per_pixel) + u64::from(offset)
                <= u64::from(transfer.1),
            "past the transfer buffer"
        );
        let region = SDL_GPUTextureRegion {
            texture: texture.0,
            w: width,
            h: height,
            d: 1,
            ..Default::default()
        };
        let to = SDL_GPUTextureTransferInfo {
            transfer_buffer: transfer.0,
            offset,
            ..Default::default()
        };
        // SAFETY: our open pass; the records live through the call; the picture fits the
        // transfer buffer (checked above).
        unsafe { SDL_DownloadFromGPUTexture(self.0, &raw const region, &raw const to) };
    }

    pub fn end(self) {
        // SAFETY: our open pass, given up here.
        unsafe { SDL_EndGPUCopyPass(self.0) };
    }
}

impl RenderPass {
    #[inline]
    pub fn bind_pipeline(&self, pipeline: Pipeline) {
        // SAFETY: our open pass; the caller's pipeline.
        unsafe { SDL_BindGPUGraphicsPipeline(self.0, pipeline.0) };
    }

    pub fn bind_vertex_buffer(&self, buffer: Buffer) {
        let binding = SDL_GPUBufferBinding {
            buffer: buffer.0,
            offset: 0,
        };
        // SAFETY: our open pass; `binding` lives through the call.
        unsafe { SDL_BindGPUVertexBuffers(self.0, 0, &raw const binding, 1) };
    }

    /// `texture` sampled by `sampler` in the fragment shader's slot 0.
    #[inline]
    pub fn bind_fragment_sampler(&self, texture: Texture, sampler: Sampler) {
        let bound = SDL_GPUTextureSamplerBinding {
            texture: texture.0,
            sampler: sampler.0,
        };
        // SAFETY: our open pass; `bound` lives through the call.
        unsafe { SDL_BindGPUFragmentSamplers(self.0, 0, &raw const bound, 1) };
    }

    /// Three textures with their samplers in the fragment shader's slots 0 to 2.
    pub fn bind_fragment_samplers3(&self, bindings: [(Texture, Sampler); 3]) {
        let bound = bindings.map(|(texture, sampler)| SDL_GPUTextureSamplerBinding {
            texture: texture.0,
            sampler: sampler.0,
        });
        // SAFETY: our open pass; `bound` lives through the call.
        unsafe { SDL_BindGPUFragmentSamplers(self.0, 0, bound.as_ptr(), 3) };
    }

    /// The viewport over the target's top-left `width` by `height` pixels, depth 0 to 1.
    pub fn set_viewport(&self, width: f32, height: f32) {
        let viewport = SDL_GPUViewport {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        // SAFETY: our open pass; `viewport` lives through the call.
        unsafe { SDL_SetGPUViewport(self.0, &raw const viewport) };
    }

    #[inline]
    pub fn set_scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        let rect = SDL_Rect {
            x,
            y,
            w: width,
            h: height,
        };
        // SAFETY: our open pass; `rect` lives through the call.
        unsafe { SDL_SetGPUScissor(self.0, &raw const rect) };
    }

    /// `count` vertices from `first`, one instance. The caller sent that many to the bound
    /// vertex buffer.
    #[inline]
    pub fn draw(&self, count: u32, first: u32) {
        // SAFETY: our open pass.
        unsafe { SDL_DrawGPUPrimitives(self.0, count, 1, first, 0) };
    }

    pub fn end(self) {
        // SAFETY: our open pass, given up here.
        unsafe { SDL_EndGPURenderPass(self.0) };
    }
}

/// The records' dwords as bytes, for a transfer buffer.
pub fn words_as_bytes<const N: usize>(records: &[[u32; N]]) -> &[u8] {
    // SAFETY: dwords as bytes: same memory, a smaller alignment, every value valid.
    unsafe { std::slice::from_raw_parts(records.as_ptr().cast(), size_of_val(records)) }
}
