use crate::types::types::{DataType, TableDefinition, Value};
use std::collections::HashMap;
use std::sync::Once;
use wgpu::util::DeviceExt;
use crate::io::util::print_table;

#[derive(Debug)]
pub struct ShaderExecutor;

impl ShaderExecutor {
    async fn run(&self, wgsl: String, data: HashMap<String, Vec<Value>>, table_definition: TableDefinition) -> HashMap<String, Vec<Value>> {
        let keys: Vec<String> = data.keys().cloned().collect();
        let first_column = data.iter().next().unwrap().clone();

        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });

        let buffers: Vec<Vec<u8>> = data
            .iter()
            .map(|(key, values)| {
                let column = table_definition
                    .columns
                    .iter()
                    .find(|c| c.name == *key)
                    .expect("Column not found in table definition");

                match column.data_type {
                    DataType::Float => {
                        let float_values: Vec<f32> = values
                            .iter()
                            .map(|v| match v {
                                Value::Float(f) => *f,
                                _ => panic!("Expected Value::Float for column {}", key),
                            })
                            .collect();
                        bytemuck::cast_slice(&float_values).to_vec()
                    }
                    DataType::Integer => {
                        let int_values: Vec<i32> = values
                            .iter()
                            .map(|v| match v {
                                Value::Integer(i) => *i,
                                _ => panic!("Expected Value::Integer for column {}", key),
                            })
                            .collect();

                        bytemuck::cast_slice(&int_values).to_vec()
                    }
                    DataType::Text | DataType::Boolean => {
                        panic!("Unsupported data type for GPU buffers: {}", key)
                    }
                }
            })
            .collect();

        let storage_staging_buffs: Vec<(wgpu::Buffer, wgpu::Buffer)> = buffers
            .iter()
            .map(|v| {
                let buffer_size = v.len() as u64;
                let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&v[..]),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                });

                let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: buffer_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });

                assert_eq!(
                    storage_buffer.size(),
                    staging_buffer.size(),
                    "Storage and staging buffer sizes must match!"
                );

                (storage_buffer, staging_buffer)
            })
            .collect();

        let storage_binding_entries: Vec<wgpu::BindGroupEntry> = storage_staging_buffs
            .iter()
            .enumerate()
            .map(|(index, (storage_buffer, _staging_buffer))| wgpu::BindGroupEntry {
                binding: index as u32,
                resource: storage_buffer.as_entire_binding(),
            })
            .collect();

        let bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = (0..storage_binding_entries.len())
            .map(|index| wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &bind_group_layout_entries,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &storage_binding_entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let total_rows = first_column.1.len();
            let workgroup_size = 64;
            let num_workgroups = (total_rows + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }
        queue.submit(Some(command_encoder.finish()));

        let mut outputs: Vec<Vec<f32>> = storage_staging_buffs
            .iter()
            .map(|(storage_buffer, _)| {
                vec![0.0; (storage_buffer.size() / size_of::<f32>() as u64) as usize]
            })
            .collect();

        self.get_data(
            &mut outputs,
            &storage_staging_buffs,
            &device,
            &queue,
        ).await;

        let mut updated_data = HashMap::new();
        for (key, output) in keys.into_iter().zip(outputs.into_iter()) {
            let values = output.into_iter().map(Value::Float).collect();
            updated_data.insert(key, values);
        }

        updated_data
    }

    async fn get_data<T: bytemuck::Pod>(
        &self,
        outputs: &mut [Vec<T>],
        storage_staging_buffs: &[(wgpu::Buffer, wgpu::Buffer)],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        for (output, (storage_buffer, staging_buffer)) in outputs.iter_mut().zip(storage_staging_buffs) {
            let mut command_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            command_encoder.copy_buffer_to_buffer(
                storage_buffer,
                0,
                staging_buffer,
                0,
                (output.len() * size_of::<T>()) as u64,
            );
            queue.submit(Some(command_encoder.finish()));
            let buffer_slice = staging_buffer.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
            device.poll(wgpu::Maintain::Wait);
            receiver.recv_async().await.unwrap().unwrap();
            output.copy_from_slice(bytemuck::cast_slice(&buffer_slice.get_mapped_range()[..]));
            staging_buffer.unmap();
        }
    }

    pub fn main(
        &self,
        wgsl: String,
        data: HashMap<String, Vec<Value>>,
        table_definition: TableDefinition,
    ) -> HashMap<String, Vec<Value>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            static INIT: Once = Once::new();
            INIT.call_once(|| {
                env_logger::builder()
                    .filter_level(log::LevelFilter::Error)
                    .format_timestamp_nanos()
                    .init();
            });

            pollster::block_on(self.run(wgsl, data, table_definition))
        }

        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));

            static INIT: Once = Once::new();
            INIT.call_once(|| {
                console_log::init_with_level(log::Level::Error).expect("could not initialize logger");
            });

            crate::utils::add_web_nothing_to_see_msg();

            // In WASM, you cannot block on a future, so the function needs to be adjusted
            // to be async if you need the result.
            wasm_bindgen_futures::spawn_local(async {
                self.run(wgsl, data, table_definition).await;
            });

            // Return an empty HashMap or handle WASM differently since it doesn't
            // allow blocking and returning the result directly.
            HashMap::new() // Placeholder for WASM
        }
    }
}