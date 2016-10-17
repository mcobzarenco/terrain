use std::collections::{VecDeque, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use chan::{self, Receiver, Sender};
use glium::index::PrimitiveType;
use glium::{IndexBuffer, VertexBuffer};
use lru_time_cache::LruCache;
use ncollide::shape::{ShapeHandle, TriMesh};
use nalgebra::{Isometry3, Point3, Translation};
use num::Zero;
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use gfx::{marching_cubes, BarycentricVertex, Camera, Mesh, Window};
use math::{GpuScalar, Vec3f, ScalarField};

pub struct LevelOfDetail<'a, Field>
    where Field: ScalarField
{
    chunk_renderer: ChunkRenderer<'a, Field>,
    octree: Octree,
    max_level: u8,
    step: f32,
}

impl<'a, Field: 'static + ScalarField + Send + Sync> LevelOfDetail<'a, Field> {
    pub fn new(scalar_field: Arc<Field>,
               thread_pool: &'a ThreadPool,
               window: &'a Window,
               max_level: u8,
               step: f32,
               size: f32,
               uid_start: usize)
               -> Self {
        LevelOfDetail {
            chunk_renderer: ChunkRenderer::new(scalar_field.clone(),
                                               thread_pool,
                                               window,
                                               uid_start),
            octree: Octree::new(Vec3f::zero() - size / 2.0, size),
            max_level: max_level,
            step: step,
        }
    }

    pub fn update(&mut self, camera: &Camera) -> Result<Vec<&Chunk>> {
        let (draw_chunk_ids, fetch_chunk_ids) = self.octree
            .rebuild(self.max_level,
                     Vec3f::from(camera.position().translation()),
                     &mut self.chunk_renderer);
        self.chunk_renderer.render(&draw_chunk_ids, fetch_chunk_ids)
    }
}

pub struct Chunk {
    pub uid: usize,
    pub tri_mesh: TriMeshHandle,
    pub index_buffer: IndexBuffer<u32>,
    pub vertex_buffer: VertexBuffer<BarycentricVertex>,
}

impl Chunk {
    fn new(uid: usize,
           window: &Window,
           mesh: Mesh<BarycentricVertex>,
           tri_mesh: TriMeshHandle)
           -> Result<Self> {
        let vertex_buffer = try!(VertexBuffer::new(window.facade(), &mesh.vertices)
            .chain_err(|| "Cannot create vertex buffer."));
        let index_buffer =
            try!(IndexBuffer::new(window.facade(), PrimitiveType::TrianglesList, &mesh.indices)
                .chain_err(|| "Cannot create index buffer."));

        Ok(Chunk {
            uid: uid,
            tri_mesh: tri_mesh,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        })
    }
}

fn field_to_mesh<Field>(scalar_field: &Field,
                        position: Vec3f,
                        size: f32,
                        step: f32,
                        iso_value: f32)
                        -> Result<Mesh<BarycentricVertex>>
    where Field: ScalarField
{
    let time = Instant::now();
    let p = position + size;
    let mesh = marching_cubes(scalar_field, &position, &p, step, iso_value)
        .with_barycentric_coordinates();
    let elapsed = time.elapsed();
    let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
    info!("Took {:.2}s to create chunk at {:?} (size {:?}) from field ({:?} vertices)",
          delta,
          position,
          size,
          mesh.vertices.len());
    Ok(mesh)
}

struct Octree {
    nodes: Vec<OctreeNode>,
    node_stack: VecDeque<usize>,
    root: OctreeNode,
}

impl Octree {
    pub fn new(position: Vec3f, size: f32) -> Self {
        let octree = Octree {
            nodes: vec![],
            node_stack: VecDeque::with_capacity(64),
            root: OctreeNode::new(position, size, 0, true),
        };
        octree
    }

    fn rebuild<Cache>(&mut self,
                      max_level: u8,
                      focus: Vec3f,
                      chunk_cache: &mut Cache)
                      -> (Vec<ChunkId>, Vec<ChunkId>)
        where Cache: ChunkCache
    {
        let Octree { ref mut nodes, ref mut node_stack, ref root } = *self;

        assert!(node_stack.is_empty());
        nodes.clear();
        nodes.push(root.clone());
        node_stack.push_back(0);
        Octree::extend_node(node_stack, nodes, max_level, focus, chunk_cache);

        let mut draw_chunk_ids = vec![];
        let mut fetch_chunk_ids = vec![];

        for node in nodes.iter() {
            if node.draw {
                draw_chunk_ids.push(node.chunk_id);
            }

            if chunk_cache.is_unknown(&node.chunk_id) {
                fetch_chunk_ids.push(node.chunk_id);
            }
        }
        (draw_chunk_ids, fetch_chunk_ids)
    }

    fn extend_node<Cache>(node_stack: &mut VecDeque<usize>,
                          nodes: &mut Vec<OctreeNode>,
                          max_level: u8,
                          focus: Vec3f,
                          chunk_cache: &mut Cache)
        where Cache: ChunkCache
    {
        while !node_stack.is_empty() {
            let current_index = node_stack.pop_front().expect("unexpected empty node stack");
            let OctreeNode { size, position, chunk_id, level, .. } = nodes[current_index];

            let is_available = chunk_cache.is_available(&chunk_id);
            if !is_available || level >= max_level ||
               distance_to_cube(&position, size, &focus) > size {
                if !is_available {
                    nodes[current_index].draw = false;
                }
                // info!("Skipping chunk {:?} with state {:?} (distance={:?})",
                //       chunk_id,
                //       chunk_cache.get_chunk_state(&chunk_id),
                //       distance_to_cube(&position, size, &focus));
            } else {
                let first_child_index = nodes.len();
                nodes[current_index].children =
                    Some(Octree::new_children_indices(first_child_index));
                let (children_positions, child_size) = Octree::children_positions(&position, size);
                for (num_child, &child_position) in children_positions.iter().enumerate() {
                    nodes.push(OctreeNode::new(child_position, child_size, level + 1, false));
                    node_stack.push_back(nodes[current_index].children.unwrap()[num_child]);
                }
                let draw_children = if nodes[current_index].draw {
                    let missing_child = nodes[current_index]
                        .children
                        .unwrap()
                        .iter()
                        .any(|child_index| {
                            !(chunk_cache.is_available(&nodes[*child_index].chunk_id) ||
                              chunk_cache.is_empty(&nodes[*child_index].chunk_id))
                        });
                    !missing_child
                } else {
                    false
                };
                if draw_children {
                    nodes[current_index].draw = false;

                    let children = nodes[current_index].children.unwrap();
                    for child_index in children.iter() {
                        nodes[*child_index].draw = true;
                    }
                }
            }
        }
    }

    #[inline]
    fn new_children_indices(next_index: usize) -> [usize; 8] {
        [next_index,
         next_index + 1,
         next_index + 2,
         next_index + 3,
         next_index + 4,
         next_index + 5,
         next_index + 6,
         next_index + 7]
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
    chunk_id: ChunkId,
    children: Option<[usize; 8]>,
    draw: bool,
}

impl OctreeNode {
    fn new(position: Vec3f, size: f32, level: u8, draw: bool) -> Self {
        OctreeNode {
            position: position,
            size: size,
            level: level,
            chunk_id: ChunkId::new(&position, size),
            children: None,
            draw: draw,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct ChunkId(i32, i32, i32, u32);

impl ChunkId {
    #[inline]
    fn new(position: &Vec3f, size: f32) -> Self {
        ChunkId((position[0] * OCTREE_VOXEL_DENSITY).floor() as i32,
                (position[1] * OCTREE_VOXEL_DENSITY).floor() as i32,
                (position[2] * OCTREE_VOXEL_DENSITY).floor() as i32,
                (size * OCTREE_VOXEL_DENSITY) as u32)
    }

    #[inline]
    pub fn position(&self) -> Vec3f {
        Vec3f::new(self.0 as f32 / OCTREE_VOXEL_DENSITY,
                   self.1 as f32 / OCTREE_VOXEL_DENSITY,
                   self.2 as f32 / OCTREE_VOXEL_DENSITY)
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.3 as f32 / OCTREE_VOXEL_DENSITY
    }
}

const OCTREE_VOXEL_DENSITY: f32 = 8.0;
const OCTREE_OFFSETS: [(f32, f32, f32); 8] = [(0.0, 0.0, 0.0),
                                              (0.0, 0.0, 1.0),
                                              (0.0, 1.0, 0.0),
                                              (1.0, 0.0, 0.0),
                                              (0.0, 1.0, 1.0),
                                              (1.0, 0.0, 1.0),
                                              (1.0, 1.0, 0.0),
                                              (1.0, 1.0, 1.0)];

#[inline]
fn distance_to_cube(cube_position: &Vec3f, size: f32, query: &Vec3f) -> f32 {
    let dx = (cube_position[0] - query[0]).max(0.0).max(query[0] - cube_position[0] - size);
    let dy = (cube_position[1] - query[1]).max(0.0).max(query[1] - cube_position[1] - size);
    let dz = (cube_position[2] - query[2]).max(0.0).max(query[2] - cube_position[2] - size);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

type TriMeshHandle = ShapeHandle<Point3<GpuScalar>, Isometry3<GpuScalar>>;
type ChunkRendererWork = (ChunkId, Mesh<BarycentricVertex>, TriMeshHandle);

struct ChunkRenderer<'a, Field: ScalarField> {
    scalar_field: Arc<Field>,
    thread_pool: &'a ThreadPool,
    window: &'a Window,
    chunk_send: Sender<ChunkRendererWork>,
    chunk_recv: Receiver<ChunkRendererWork>,
    loaded_chunks: LruCache<ChunkId, Chunk>,
    pending_chunks: HashSet<ChunkId>,
    empty_chunks: LruCache<ChunkId, ()>,
    empty_uid: usize,
}

impl<'a, Field> ChunkRenderer<'a, Field>
    where Field: 'static + ScalarField + Send + Sync
{
    fn new(scalar_field: Arc<Field>,
           thread_pool: &'a ThreadPool,
           window: &'a Window,
           uid_start: usize)
           -> Self {
        let (send, recv) = chan::sync(128);
        ChunkRenderer {
            scalar_field: scalar_field,
            thread_pool: thread_pool,
            window: window,
            chunk_send: send,
            chunk_recv: recv,
            loaded_chunks: LruCache::with_capacity(2048),
            pending_chunks: HashSet::with_capacity(128),
            empty_chunks: LruCache::with_capacity(65536),
            empty_uid: uid_start,
        }
    }

    fn render(&mut self,
              draw_chunk_ids: &Vec<ChunkId>,
              fetch_chunk_ids: Vec<ChunkId>)
              -> Result<Vec<&Chunk>> {

        // The invariant required to hold when calling this function is:
        //   - the meshes for all `draw_chunk_ids` are available
        //   - the meshes for all `fetch_chunk_ids` are unknown
        //
        // A mesh with `chunk_id` is defined to be available iff
        //     `get_chunk_state(&chunk_id) == ChunkState::Available`
        // println!("draw: {:?}", draw_chunk_ids);

        assert!(draw_chunk_ids.iter()
            .all(|chunk_id| self.get_chunk_state(chunk_id) == ChunkState::Available));
        assert!(fetch_chunk_ids.iter()
            .all(|chunk_id| self.get_chunk_state(chunk_id) == ChunkState::Unknown));

        let ChunkRenderer { ref scalar_field,
                            ref thread_pool,
                            ref chunk_send,
                            ref chunk_recv,
                            ref mut loaded_chunks,
                            ref mut pending_chunks,
                            ref mut empty_chunks,
                            .. } = *self;

        while let Some((chunk_id, mesh, tri_mesh)) = (|| {
            chan_select! {
                default => { return None; },
                chunk_recv.recv() -> maybe_chunk => { return maybe_chunk; },
            }
        })() {
            info!("Received chunk with {} vertices.", mesh.vertices.len());
            pending_chunks.remove(&chunk_id);
            if mesh.vertices.len() > 0 {
                loaded_chunks.insert(chunk_id,
                                     try!(Chunk::new(self.empty_uid, self.window, mesh, tri_mesh)));
                self.empty_uid += 1;
            } else {
                empty_chunks.insert(chunk_id, ());
            }
        }

        for chunk_id in fetch_chunk_ids.into_iter() {
            if pending_chunks.len() > 8 {
                break;
            }

            info!("Submitted chunk {:?}.", chunk_id);
            let position = chunk_id.position();
            let chunk_size = chunk_id.size();

            let num_steps = 16.0;
            let step_size = chunk_size / num_steps;
            let scalar_field = scalar_field.clone();
            let sender = chunk_send.clone();
            thread_pool.execute(move || {
                let mesh = field_to_mesh(scalar_field.deref(),
                                         position,
                                         chunk_size + step_size,
                                         step_size,
                                         0.0)
                    .unwrap();
                let tri_mesh =
                    TriMesh::new(Arc::new(mesh.vertices
                                     .iter()
                                     .map(|x| x.position.to_point())
                                     .collect()),
                                 Arc::new(mesh.indices
                                     .chunks(3)
                                     .map(|x| {
                                         Point3::new(x[0] as usize, x[1] as usize, x[2] as usize)
                                     })
                                     .collect()),
                                 None,
                                 None);

                // info!("Chunk: {:?}", chunk);
                sender.send((chunk_id, mesh, ShapeHandle::new(tri_mesh)));
            });
            pending_chunks.insert(chunk_id);
        }

        let mut draw_chunks = vec![];
        for chunk_id in draw_chunk_ids.iter() {
            if let Some(chunk) = loaded_chunks.peek(chunk_id) {
                draw_chunks.push(chunk);
            } else {
                warn!("A chunk needed to be drawn was evicted after collecting new chunks from \
                       workers, increase the LRU chunk cache size.");
            }
        }

        Ok(draw_chunks)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ChunkState {
    Unknown, // The chunk's mesh has not been computed
    Pending, // The chunk's mesh is being computed
    Empty, // The chunk's mesh does not contain any vertices
    Available, // The chunk's mesh is available to draw
}

trait ChunkCache {
    #[inline]
    fn get_chunk_state(&mut self, chunk_id: &ChunkId) -> ChunkState;

    #[inline]
    fn is_unknown(&mut self, chunk_id: &ChunkId) -> bool {
        self.get_chunk_state(chunk_id) == ChunkState::Unknown
    }

    #[inline]
    fn is_empty(&mut self, chunk_id: &ChunkId) -> bool {
        self.get_chunk_state(chunk_id) == ChunkState::Empty
    }

    #[inline]
    fn is_available(&mut self, chunk_id: &ChunkId) -> bool {
        self.get_chunk_state(chunk_id) == ChunkState::Available
    }
}

impl<'a, Field> ChunkCache for ChunkRenderer<'a, Field>
    where Field: 'static + ScalarField + Send + Sync
{
    #[inline]
    fn get_chunk_state(&mut self, chunk_id: &ChunkId) -> ChunkState {
        if self.loaded_chunks.get(chunk_id).is_some() {
            assert!(!self.empty_chunks.contains_key(chunk_id) &&
                    !self.pending_chunks.contains(chunk_id));
            ChunkState::Available
        } else if self.empty_chunks.contains_key(chunk_id) {
            assert!(!self.pending_chunks.contains(chunk_id));
            ChunkState::Empty
        } else if self.pending_chunks.contains(chunk_id) {
            ChunkState::Pending
        } else {
            ChunkState::Unknown
        }
    }
}
