use std::error::Error;
use wasmtime::*;
use wgpu::{
    include_wgsl, CommandBuffer, CommandEncoderDescriptor, Device, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, SurfaceConfiguration, SurfaceTexture, TextureFormat,
    TextureView,
};

pub mod basetag;

/// TODO: This should include the previous tag tree
#[allow(dead_code)]
pub struct ChipState {
    pipeline: RenderPipeline,
}

impl ChipState {
    /// TODO: wasm
    /// This loads/parses the shader, creates a pipeline, and compiles/loads all wasm (not currently, that is)
    pub fn setup(device: &Device, format: TextureFormat) -> Result<ChipState, Box<dyn Error>> {
        let shader = device.create_shader_module(&include_wgsl!("chip-shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Chip Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Chip Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(ChipState { pipeline })
    }

    pub fn render(
        &mut self,
        device: &Device,
        view: &TextureView,
    ) -> Result<CommandBuffer, Box<dyn Error>> {
        // update(state)?;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Chip Render Encoder"),
        });

        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Chip Render Pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        drop(render_pass);

        Ok(encoder.finish())
    }

    pub fn reset(&mut self) {}

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let engine = Engine::default();
        let module = Module::new_with_name(&engine, "(module (func))", "test-module")?;
        let mut linker = Linker::new(&engine);
        linker.func_wrap("test-module", "double", |param: i32| param * 2)?;
        // linker.func_wrap(
        //     "test-module",
        //     "log",
        //     |mut caller: Caller<'_, Log>, param: u32| {
        //         println!("log: {}", param);
        //         caller.data_mut().integers_logged.push(param);
        //     },
        // )?;

        let mut store = Store::new(&engine, /* data */ ());
        let instance = linker.instantiate(&mut store, &module)?;

        let answer = instance
            .get_typed_func::<(), i32, _>(&mut store, "answer")
            .expect("`answer` was not an exported function");

        let result = answer.call(&mut store, ())?;

        // let log = store.data().integers_logged;

        println!("{:?}", result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn serialize() {
        use crate::basetag::*;
        let basetag = Tag {
            style_type: StyleType::Flex,
            style_base: StyleBase {
                width: 10.0,
                height: 10.0,
                border_radius: 0.0,
                stroke_width: 1.0,
                fill_color: [255, 255, 255, 255],
                outline_color: [0, 0, 0, 255],
            },
            children: vec![Tag {
                style_type: StyleType::Table {
                    size_method: TableSizeMethod::Auto,
                    columns: 3,
                    rows: 2,
                },
                style_base: StyleBase {
                    width: 200.0,
                    height: 20.0,
                    border_radius: 15.0,
                    stroke_width: 2.0,
                    fill_color: [255, 0, 255, 127],
                    outline_color: [20, 200, 50, 200],
                },
                children: vec![],
            }],
        };
        let encoded = bincode::serialize(&basetag).unwrap();
        // This is:
        // 4 bytes for StyleType::Flex,
        // 4 * 4 bytes for the four f32's in StyleBase
        // 2 * 4 bytes for the colors in StyleBase
        // 8 bytes for the len of the vec
        // 48 bytes for the other tag; see `small_serialize`
        assert_eq!(encoded.len(), 84);
        let decoded: Tag = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(basetag, decoded);
    }

    #[test]
    fn small_serialize() {
        use crate::basetag::*;
        let tag = Tag {
            style_type: StyleType::Table {
                size_method: TableSizeMethod::Auto,
                columns: 3,
                rows: 2,
            },
            style_base: StyleBase {
                width: 200.0,
                height: 20.0,
                border_radius: 15.0,
                stroke_width: 2.0,
                fill_color: [255, 0, 255, 127],
                outline_color: [20, 200, 50, 200],
            },
            children: vec![],
        };
        let encoded = bincode::serialize(&tag).unwrap();
        // This is:
        // 4 bytes for StyleType::Table
        // 4 bytes for TableSizeMethod::Auto,
        // 2 * 4 bytes for the column/row
        // 6 * 4 bytes for StyleBase
        // 8 bytes for the vec
        assert_eq!(encoded.len(), 48);
        let decoded: Tag = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(tag, decoded);
    }

    #[test]
    fn serialize_parts() {
        use crate::basetag::*;
        let style_type = StyleType::Table {
            size_method: TableSizeMethod::Auto,
            columns: 3,
            rows: 2,
        };
        let encoded = bincode::serialize(&style_type).unwrap();
        // This is:
        // 4 bytes for StyleTable::Table
        // 4 bytes for size_method
        // 2 * 4 bytes for the u32's
        assert_eq!(encoded.len(), 16);
        let decoded: StyleType = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(decoded, style_type);
    }
}
