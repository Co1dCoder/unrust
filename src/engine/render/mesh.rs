use webgl::*;
use std::mem::size_of;

use super::ShaderProgram;
use engine::core::ComponentBased;
use engine::asset::{Asset, AssetError, AssetSystem, File, FileFuture, LoadableAsset, Resource};
use engine::asset::loader;
use engine::asset::loader::Loadable;

use std::cell::RefCell;
use std::rc::Rc;
use std::f32::{MAX, MIN};

use na::Vector3;

trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T> IntoBytes for Vec<T> {
    fn into_bytes(self) -> Vec<u8> {
        let len = size_of::<T>() * self.len();
        unsafe {
            let slice = self.into_boxed_slice();
            Vec::<u8>::from_raw_parts(Box::into_raw(slice) as _, len, len)
        }
    }
}

pub struct Mesh {
    pub mesh_buffer: Rc<MeshBuffer>,
}

impl ComponentBased for Mesh {}

struct MeshGLState {
    pub vb: WebGLBuffer,
    pub uvb: Option<WebGLBuffer>,
    pub nb: Option<WebGLBuffer>,
    pub ib: WebGLBuffer,
}

impl Mesh {
    pub fn new(mesh_buffer: Rc<MeshBuffer>) -> Mesh {
        Mesh {
            mesh_buffer: mesh_buffer,
        }
    }

    pub fn bind(
        &self,
        gl: &WebGLRenderingContext,
        program: &ShaderProgram,
    ) -> Result<(), AssetError> {
        self.mesh_buffer.bind(gl, program)
    }

    pub fn render(&self, gl: &WebGLRenderingContext) {
        let data = self.mesh_buffer.data.try_borrow().unwrap();

        gl.draw_elements(Primitives::Triangles, data.indices.len(), DataType::U16, 0);
    }
}

#[derive(Default, Debug)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub uvs: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub indices: Vec<u16>,
}

pub struct MeshLoader {}
impl loader::Loader<MeshData> for MeshLoader {
    fn load(_file: Box<File>) -> Result<MeshData, AssetError> {
        unimplemented!()
    }
}

impl Loadable for MeshData {
    type Loader = MeshLoader;
}

pub struct MeshBuffer {
    data: Resource<MeshData>,
    gl_state: RefCell<Option<MeshGLState>>,
    bounds: RefCell<Option<(Vector3<f32>, Vector3<f32>)>>,
}

impl Asset for MeshBuffer {
    type Resource = Resource<MeshData>;

    fn new_from_resource(r: Self::Resource) -> Rc<Self> {
        Rc::new(MeshBuffer {
            data: r,
            gl_state: Default::default(),
            bounds: Default::default(),
        })
    }
}

impl LoadableAsset for MeshBuffer {
    fn load<T: AssetSystem>(_asys: &T, mut files: Vec<FileFuture>) -> Self::Resource {
        Self::load_resource::<MeshData>(files.remove(0))
    }

    fn gather<T: AssetSystem>(asys: &T, fname: &str) -> Vec<FileFuture> {
        vec![asys.new_file(fname)]
    }
}

impl MeshBuffer {
    pub fn prepare(&self, gl: &WebGLRenderingContext) -> Result<(), AssetError> {
        if self.gl_state.borrow().is_some() {
            return Ok(());
        }
        let data = self.data.try_borrow()?;

        self.gl_state.replace(Some(mesh_bind_buffer(
            &data.vertices,
            &data.uvs,
            &data.normals,
            &data.indices,
            gl,
        )));

        Ok(())
    }

    fn compute_bounds(&self) -> Option<(Vector3<f32>, Vector3<f32>)> {
        let mut min = Vector3::new(MAX, MAX, MAX);
        let mut max = Vector3::new(MIN, MIN, MIN);

        let data = self.data.try_borrow().ok()?;

        for (i, v) in data.vertices.iter().enumerate() {
            min[i % 3] = v.min(min[i % 3]);
            max[i % 3] = v.max(max[i % 3]);
        }

        Some((min, max))
    }

    /// bounds return (vmin, vmax)
    pub fn bounds(&self) -> Option<(Vector3<f32>, Vector3<f32>)> {
        let mut bounds = self.bounds.borrow_mut();

        match *bounds {
            Some(ref k) => Some(*k),
            None => {
                *bounds = self.compute_bounds();
                *bounds
            }
        }
    }

    pub fn bind(
        &self,
        gl: &WebGLRenderingContext,
        program: &ShaderProgram,
    ) -> Result<(), AssetError> {
        self.prepare(gl)?;

        let state_option = self.gl_state.borrow();
        let state = state_option.as_ref().unwrap();

        /*======= Associating shaders to buffer objects =======*/

        // Bind vertex buffer object
        gl.bind_buffer(BufferKind::Array, &state.vb);

        // Point an position attribute to the currently bound VBO
        if let Some(coord) = program.attrib_loc(gl, "aVertexPosition") {
            gl.enable_vertex_attrib_array(coord);
            gl.vertex_attrib_pointer(coord, AttributeSize::Three, DataType::Float, false, 0, 0);
        }

        if let Some(ref nb) = state.nb {
            gl.bind_buffer(BufferKind::Array, nb);
            // Point an normal attribute to the currently bound VBO

            if let Some(coord) = program.attrib_loc(gl, "aVertexNormal") {
                gl.enable_vertex_attrib_array(coord);
                gl.vertex_attrib_pointer(coord, AttributeSize::Three, DataType::Float, false, 0, 0);
            }
        }

        if let Some(ref uvb) = state.uvb {
            gl.bind_buffer(BufferKind::Array, uvb);
            // Point an uv attribute to the currently bound VBO

            if let Some(coord) = program.attrib_loc(gl, "aTextureCoord") {
                gl.enable_vertex_attrib_array(coord);
                gl.vertex_attrib_pointer(coord, AttributeSize::Two, DataType::Float, false, 0, 0);
            }
        }

        // Bind index buffer object
        gl.bind_buffer(BufferKind::ElementArray, &state.ib);

        Ok(())
    }
}

fn mesh_bind_buffer(
    vertices: &Vec<f32>,
    uvs: &Option<Vec<f32>>,
    normals: &Option<Vec<f32>>,
    indices: &Vec<u16>,
    gl: &WebGLRenderingContext,
) -> MeshGLState {
    // Create an empty buffer object to store vertex buffer
    let vertex_buffer = gl.create_buffer();
    {
        // Bind appropriate array buffer to it
        gl.bind_buffer(BufferKind::Array, &vertex_buffer);

        // Pass the vertex data to the buffer
        let cv = vertices.clone();
        gl.buffer_data(BufferKind::Array, &cv.into_bytes(), DrawMode::Static);

        // Unbind the buffer
        gl.unbind_buffer(BufferKind::Array);
    }

    // Create an empty buffer object to store uv buffer
    let uv_buffer = match uvs {
        &Some(ref uvs) => {
            let uv_buffer = gl.create_buffer();
            {
                // Bind appropriate array buffer to it
                gl.bind_buffer(BufferKind::Array, &uv_buffer);

                // Pass the vertex data to the buffer
                let uvv = uvs.clone();
                gl.buffer_data(BufferKind::Array, &uvv.into_bytes(), DrawMode::Static);

                // Unbind the buffer
                gl.unbind_buffer(BufferKind::Array);

                Some(uv_buffer)
            }
        }
        _ => None,
    };

    // Create an Normal Buffer
    let normal_buffer = match normals {
        &Some(ref normals) => {
            let normal_buffer = gl.create_buffer();
            {
                // Bind appropriate array buffer to it
                gl.bind_buffer(BufferKind::Array, &normal_buffer);

                let ns = normals.clone();
                gl.buffer_data(BufferKind::Array, &ns.into_bytes(), DrawMode::Static);

                // Unbind the buffer
                gl.unbind_buffer(BufferKind::Array);

                Some(normal_buffer)
            }
        }
        _ => None,
    };

    // Create an empty buffer object to store Index buffer
    let index_buffer = gl.create_buffer();
    {
        // Bind appropriate array buffer to it
        gl.bind_buffer(BufferKind::ElementArray, &index_buffer);

        // Pass the vertex data to the buffer
        let ci = indices.clone();
        gl.buffer_data(BufferKind::ElementArray, &ci.into_bytes(), DrawMode::Static);

        // Unbind the buffer
        gl.unbind_buffer(BufferKind::ElementArray);
    }

    MeshGLState {
        vb: vertex_buffer,
        uvb: uv_buffer,
        nb: normal_buffer,
        ib: index_buffer,
    }
}
