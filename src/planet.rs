use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use chan::{self, Receiver, Sender};
use glium::{self, Frame, DrawParameters, IndexBuffer, Program, Surface, VertexBuffer};
use glium::backend::glutin_backend::GlutinFacade;
use glium::index::PrimitiveType;
use noise::{self, Seed, Brownian3};
use num::{Float, Zero};
use threadpool::ThreadPool;
use lru_time_cache::LruCache;

use errors::{ChainErr, Result};
use gfx::{marching_cubes, Camera, Mesh, Vertex};
use math::{Vec3f, Vector, ScalarField};
use utils::read_utf8_file;

pub struct TerrainField {
    seed: Seed,
}

impl TerrainField {
    pub fn new(seed: u32) -> Self {
        TerrainField { seed: Seed::new(seed) }
    }
}

impl ScalarField for TerrainField {
    #[inline]
    fn value_at(&self, x: f32, y: f32, z: f32) -> f32 {
        // let x = x + noise::perlin3(&self.seed, &[y, z, x]) / 5.0;
        // let y = y + noise::perlin3(&self.seed, &[z, x, y]) / 5.0;
        // let z = z + noise::perlin3(&self.seed, &[x, y, z]) / 5.0;

        let scale = 45.0;
        let mut pos = Vec3f::new(x / scale, y / scale, z / scale);
        let distance = pos.norm();
        pos.normalize();

        let noise = Brownian3::new(noise::open_simplex3, 11)
            .persistence(0.753)
            .wavelength(2.1)
            .lacunarity(1.752);
        let height = noise.apply(&self.seed, &pos.array()) / 2.0;

        // let a = 0.45;
        // sample * 1.0 / (1.0 + scale * x.norm())
        // println!("{}", sample);
        distance + height
        // if distance < height {
        //     // let noise = Brownian3::new(noise::open_simplex3, 3)
        //     //     .persistence(0.7)
        //     //     .wavelength(10.0)
        //     //     .lacunarity(2.0);
        //     // let a = (1.5 + noise.apply(&self.seed, &[y, x, z])) / 2.5;
        //     // println!("{}", distance);
        //     // a / (distance + 1e-2)
        //     // (0.5 + height - distance) * a

        //     1.0
        // } else {
        //     0.0
        // }
    }
}

struct Octree {
    nodes: Vec<OctreeNode>,
    node_stack: Vec<usize>,
}

impl Octree {
    pub fn new(position: Vec3f, size: f32) -> Self {
        let mut octree = Octree {
            nodes: vec![OctreeNode::new(position, size, 0)],
            node_stack: vec![],
        };
        octree.rebuild(0, position);
        octree
    }

    fn rebuild(&mut self, max_level: u8, focus: Vec3f) -> Vec<ChunkId> {
        assert!(self.node_stack.is_empty());
        self.nodes.truncate(1);
        self.node_stack.push(0);
        Octree::extend_node(&mut self.node_stack, &mut self.nodes, max_level, focus);

        let mut chunk_ids = vec![];
        for node in self.nodes.iter() {
            // if node.children.iter().all(Option::is_some) {
            //     if let Some(chunk_id) = node.chunk {
            //         chunk_ids.push(chunk_id);
            //     }
            // }

            if node.children == [None; 8] {
                if let Some(chunk_id) = node.chunk {
                    chunk_ids.push(chunk_id);
                }
            }
        }
        chunk_ids
    }

    fn extend_node(node_stack: &mut Vec<usize>,
                   nodes: &mut Vec<OctreeNode>,
                   max_level: u8,
                   focus: Vec3f) {
        while !node_stack.is_empty() {
            // println!("stack: {:?}", node_stack);
            // println!(" ^ current node: {:?}",
            //          nodes[node_stack[node_stack.len() - 1]]);
            let current_index = node_stack.pop().expect("unexpected empty node stack");
            let OctreeNode { size, position, level, .. } = nodes[current_index];
            let chunk_id = Octree::chunk_id(&position, size);
            nodes[current_index].chunk = Some(chunk_id);
            // println!("level: {:?} / {} - {:?}", level, max_level, chunk_id);

            if size <= 1.0 || level >= max_level ||
               (position + (size / 2.0) - focus).norm() > 4.0.max(size) {
            } else {
                let (children_positions, child_size) = Octree::children_positions(&position, size);
                for (child_index, &child_position) in children_positions.into_iter().enumerate() {
                    nodes.push(OctreeNode::new(child_position, child_size, level + 1));
                    let child_global_index = nodes.len() - 1;
                    nodes[current_index].children[child_index] = Some(child_global_index);
                    node_stack.push(child_global_index);
                }
            }
        }
    }

    #[inline]
    fn chunk_id(position: &Vec3f, size: f32) -> ChunkId {
        (position[0].floor() as i32,
         position[1].floor() as i32,
         position[2].floor() as i32,
         size as u32)
    }

    #[inline]
    fn children_positions(position: &Vec3f, size: f32) -> ([Vec3f; 8], f32) {
        let child_size = size / 2.0;
        let make_position = |position: &Vec3f, offset: (f32, f32, f32)| -> Vec3f {
            Vec3f::new(position[0] + child_size * offset.0,
                       position[1] + child_size * offset.1,
                       position[2] + child_size * offset.2)
        };
        let positions = [make_position(position, OCTREE_OFFSETS[0]),
                         make_position(position, OCTREE_OFFSETS[1]),
                         make_position(position, OCTREE_OFFSETS[2]),
                         make_position(position, OCTREE_OFFSETS[3]),
                         make_position(position, OCTREE_OFFSETS[4]),
                         make_position(position, OCTREE_OFFSETS[5]),
                         make_position(position, OCTREE_OFFSETS[6]),
                         make_position(position, OCTREE_OFFSETS[7])];
        (positions, child_size)
    }
}

#[derive(Clone, Debug)]
struct OctreeNode {
    position: Vec3f,
    size: f32,
    level: u8,
    chunk: Option<ChunkId>,
    children: [Option<usize>; 8],
}

impl OctreeNode {
    fn new(position: Vec3f, size: f32, level: u8) -> Self {
        OctreeNode {
            level: level,
            position: position,
            size: size,
            chunk: None,
            children: [None; 8],
        }
    }
}

type ChunkId = (i32, i32, i32, u32);
const OCTREE_OFFSETS: [(f32, f32, f32); 8] = [(0.0, 0.0, 0.0),
                                              (0.0, 0.0, 1.0),
                                              (0.0, 1.0, 0.0),
                                              (1.0, 0.0, 0.0),
                                              (0.0, 1.0, 1.0),
                                              (1.0, 0.0, 1.0),
                                              (1.0, 1.0, 0.0),
                                              (1.0, 1.0, 1.0)];

struct LevelOfDetail<'a, 'b, Field: ScalarField> {
    facade: &'a GlutinFacade,
    thread_pool: &'b ThreadPool,
    chunk_send: Sender<(ChunkId, Chunk)>,
    chunk_recv: Receiver<(ChunkId, Chunk)>,
    scalar_field: Arc<Field>,
    level: usize,
    step: f32,
    size: f32,
    chunks: LruCache<ChunkId, Option<BufferedChunk>>,
    empty_chunks: LruCache<ChunkId, ()>,
}

impl<'a, 'b, Field: 'static + ScalarField + Send + Sync> LevelOfDetail<'a, 'b, Field> {
    fn new(scalar_field: Arc<Field>,
           thread_pool: &'b ThreadPool,
           facade: &'a GlutinFacade,
           level: usize,
           step: f32,
           size: f32)
           -> Self {
        let (send, recv) = chan::sync(128);
        LevelOfDetail {
            thread_pool: thread_pool,
            facade: facade,
            chunk_send: send,
            chunk_recv: recv,
            scalar_field: scalar_field,
            level: level,
            step: step,
            size: size,
            chunks: LruCache::with_capacity(1024),
            empty_chunks: LruCache::with_capacity(65536),
        }
    }

    pub fn update<R>(&mut self, camera: &Camera, mut render: R) -> Result<()>
        where R: FnMut(&VertexBuffer<Vertex>, &IndexBuffer<u32>) -> Result<()>
    {
        let mut octree = Octree::new(Vec3f::zero() - 32.0, 64.0);
        let chunk_ids = octree.rebuild(6, camera.position);
        {
            let LevelOfDetail { ref chunk_send,
                                ref mut chunks,
                                ref mut empty_chunks,
                                ref scalar_field,
                                ref thread_pool,
                                .. } = *self;

            for &chunk_id in chunk_ids.iter().filter(|&x| !empty_chunks.contains_key(x)) {
                chunks.entry(chunk_id).or_insert_with(|| {
                    let position =
                        Vec3f::new(chunk_id.0 as f32, chunk_id.1 as f32, chunk_id.2 as f32);
                    let chunk_size = chunk_id.3 as f32;

                    let num_steps = 17.0;
                    let step_size = chunk_size / num_steps;
                    let scalar_field = scalar_field.clone();
                    let sender = chunk_send.clone();
                    thread_pool.execute(move || {
                        let chunk = Chunk::new(scalar_field.deref(),
                                               position,
                                               chunk_size + step_size,
                                               step_size,
                                               0.5)
                            .unwrap();
                        sender.send((chunk_id, chunk));
                    });
                    None
                });
            }
        }

        while let Some((chunk_id, chunk)) = (|| {
            let chunk_recv = &self.chunk_recv;

            chan_select! {
                default => { return None; },
                chunk_recv.recv() -> maybe_chunk => { return maybe_chunk; },
            }
        })() {
            info!("Received chunk with {} vertices.",
                  chunk.mesh.vertices.len());
            if chunk.mesh.vertices.len() > 0 {
                try!(self.add_chunk(chunk_id, chunk));
            } else {
                self.chunks.remove(&chunk_id);
                let mut chunk_id = chunk_id;
                self.empty_chunks.insert(chunk_id, ());
                chunk_id.3 /= 2;
                self.empty_chunks.insert(chunk_id, ());
                chunk_id.3 /= 2;
                self.empty_chunks.insert(chunk_id, ());
                chunk_id.3 /= 2;
                self.empty_chunks.insert(chunk_id, ());
                chunk_id.3 /= 2;
                self.empty_chunks.insert(chunk_id, ());
            }
        }

        for chunk_id in chunk_ids.iter() {
            if let Some(&Some(ref buffer)) = self.chunks.get(chunk_id) {
                try!(render(&buffer.vertex_buffer, &buffer.index_buffer));
            }
        }
        Ok(())
    }

    fn add_chunk(&mut self, chunk_id: ChunkId, chunk: Chunk) -> Result<()> {
        let vertex_buffer = try!(VertexBuffer::new(self.facade, &chunk.mesh.vertices)
            .chain_err(|| "Cannot create vertex buffer."));
        let index_buffer = try!(IndexBuffer::new(self.facade,
                                                 PrimitiveType::TrianglesList,
                                                 &chunk.mesh.indices)
            .chain_err(|| "Cannot create index buffer."));
        let buffer = BufferedChunk {
            chunk: chunk,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        };
        self.chunks.insert(chunk_id, Some(buffer));
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Chunk {
    mesh: Mesh,
}

impl Chunk {
    fn new<Field>(scalar_field: &Field,
                  position: Vec3f,
                  size: f32,
                  step: f32,
                  iso_value: f32)
                  -> Result<Self>
        where Field: ScalarField
    {
        let time = Instant::now();
        let p = position + size;
        let mesh = marching_cubes(scalar_field, &position, &p, step, iso_value);
        let elapsed = time.elapsed();
        let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
        info!("Took {:.2}s to create chunk at {:?} (size {:?}) from field ({:?} vertices)",
              delta,
              position,
              size,
              mesh.vertices.len());

        Ok(Chunk { mesh: mesh })
    }
}

struct BufferedChunk {
    chunk: Chunk,
    index_buffer: IndexBuffer<u32>,
    vertex_buffer: VertexBuffer<Vertex>,
}

pub struct Planet<'a, 'b, 'c, Field: ScalarField> {
    lod: LevelOfDetail<'a, 'b, Field>,
    draw_parameters: DrawParameters<'c>,
    program: Program,
}

impl<'a, 'b, 'c, Field: 'static + ScalarField + Send + Sync> Planet<'a, 'b, 'c, Field> {
    pub fn new(scalar_field: Field,
               facade: &'a GlutinFacade,
               thread_pool: &'b ThreadPool)
               -> Result<Self> {

        let vertex_shader = try!(read_utf8_file(VERTEX_SHADER));
        let fragment_shader = try!(read_utf8_file(FRAGMENT_SHADER));
        let program =
            try!(glium::Program::from_source(facade, &vertex_shader, &fragment_shader, None)
                .chain_err(|| "Could not compile the shaders."));

        let scalar_field = Arc::new(scalar_field);
        let lod = LevelOfDetail::new(scalar_field.clone(), thread_pool, facade, 0, 16.0, 16.0);

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };

        Ok(Planet {
            lod: lod,
            draw_parameters: params,
            program: program,
        })
    }

    pub fn render(&mut self, frame: &mut Frame, camera: &Camera) -> Result<()> {
        let model = [[1.0, 0.0, 0.0, 0.0],
                     [0.0, 1.0, 0.0, 0.0],
                     [0.0, 0.0, 1.0, 0.0],
                     [0.0, 0.0, 0.0, 1.0f32]];
        let view = camera.view_matrix();

        let perspective = {
            let (width, height) = frame.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = 3.141592 / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [[f * aspect_ratio, 0.0, 0.0, 0.0],
             [0.0, f, 0.0, 0.0],
             [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
             [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]]
        };
        let light = [-20.0f32, 0.0, -50.0];

        let uniforms = uniform! {
            perspective: perspective,
            model: model,
            view: view,
            u_light: light,
        };

        let program = &self.program;
        let draw_parameters = &self.draw_parameters;
        try!(self.lod.update(camera, |vertex_buffer, index_buffer| {
            frame.draw(vertex_buffer,
                      index_buffer,
                      &program,
                      &uniforms,
                      &draw_parameters)
                .chain_err(|| "Could not render frame.")
        }));

        Ok(())
    }
}

const VERTEX_SHADER: &'static str = "src/gfx/shaders/planet.vert";
const FRAGMENT_SHADER: &'static str = "src/gfx/shaders/planet.frag";
