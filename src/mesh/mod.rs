use generational_arena::{Arena, Index};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::path::Path;

use crate::drawable::Drawable;
use crate::glm;
use crate::gpu_immediate::*;
use crate::meshio::{MeshIO, MeshIOError};
use crate::shader;

pub mod builtins;

/// Node stores the world (3D) space coordinates
///
/// Each Node also optionally stores 3D space normal information
/// (commonly referred to as Vertex Normals)
///
/// Each Node can be referred to by many Verts
#[derive(Debug, Serialize, Deserialize)]
pub struct Node<T> {
    self_index: NodeIndex,
    pub pos: glm::DVec3,
    pub normal: Option<glm::DVec3>,
    pub extra_data: Option<T>,

    verts: IncidentVerts,
}

/// Vert stores the uv space coordinates
///
/// A Vert can only have one Node but this Node can be shared by many Verts
///
/// Each Vert can be referred to by many Edges
#[derive(Debug, Serialize, Deserialize)]
pub struct Vert<T> {
    self_index: VertIndex,
    pub uv: Option<glm::DVec2>,
    pub extra_data: Option<T>,

    node: Option<NodeIndex>,
    edges: IncidentEdges,
}

/// Edge stores the information gap between faces and vertices to allow for faster access of adjacent face information
///
/// Each Edge has a pair of Verts (Made as Option because it may not
/// have this information when it first is created)
#[derive(Debug, Serialize, Deserialize)]
pub struct Edge<T> {
    self_index: EdgeIndex,
    pub extra_data: Option<T>,

    verts: Option<(VertIndex, VertIndex)>,
    faces: IncidentFaces,
}

/// Face stores the vertices in order that form that face, this is done instead of storing edges to prevent winding/orientation problems with the mesh.
///
/// Each Face also stores the face normal optionally
#[derive(Debug, Serialize, Deserialize)]
pub struct Face<T> {
    self_index: FaceIndex,
    pub normal: Option<glm::DVec3>,
    pub extra_data: Option<T>,

    verts: AdjacentVerts,
}

/// Mesh stores the Node, Vert, Edge, Face data in an Arena
///
/// Mesh optionally stores a renderable mesh, GLMesh
#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh<END, EVD, EED, EFD> {
    nodes: Arena<Node<END>>,
    verts: Arena<Vert<EVD>>,
    edges: Arena<Edge<EED>>,
    faces: Arena<Face<EFD>>,
}

/// Index of Node in Mesh.nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeIndex(pub Index);
/// Index of Vert in Mesh.nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VertIndex(pub Index);
/// Index of Edge in Mesh.nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EdgeIndex(pub Index);
/// Index of Face in Mesh.nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FaceIndex(pub Index);

type IncidentVerts = Vec<VertIndex>;
type IncidentEdges = Vec<EdgeIndex>;
type IncidentFaces = Vec<FaceIndex>;
type AdjacentVerts = IncidentVerts;

/// Errors during operations on Mesh
#[derive(Debug)]
pub enum MeshError {
    MeshIO(MeshIOError),
    NoUV,
}

impl From<MeshIOError> for MeshError {
    fn from(err: MeshIOError) -> MeshError {
        MeshError::MeshIO(err)
    }
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::MeshIO(error) => write!(f, "{}", error),
            MeshError::NoUV => write!(f, "No UV information found"),
        }
    }
}

impl std::error::Error for MeshError {}

impl<END, EVD, EED, EFD> Default for Mesh<END, EVD, EED, EFD> {
    fn default() -> Self {
        Mesh::new()
    }
}

impl<END, EVD, EED, EFD> Mesh<END, EVD, EED, EFD> {
    pub fn new() -> Mesh<END, EVD, EED, EFD> {
        Mesh {
            nodes: Arena::new(),
            verts: Arena::new(),
            edges: Arena::new(),
            faces: Arena::new(),
        }
    }

    pub fn from_arenas(
        nodes: Arena<Node<END>>,
        verts: Arena<Vert<EVD>>,
        edges: Arena<Edge<EED>>,
        faces: Arena<Face<EFD>>,
    ) -> Self {
        Self {
            nodes,
            verts,
            edges,
            faces,
        }
    }

    pub fn get_faces(&self) -> &Arena<Face<EFD>> {
        &self.faces
    }

    pub fn get_edges(&self) -> &Arena<Edge<EED>> {
        &self.edges
    }

    pub fn get_verts(&self) -> &Arena<Vert<EVD>> {
        &self.verts
    }

    pub fn get_nodes(&self) -> &Arena<Node<END>> {
        &self.nodes
    }

    pub fn get_faces_mut(&mut self) -> &mut Arena<Face<EFD>> {
        &mut self.faces
    }

    pub fn get_edges_mut(&mut self) -> &mut Arena<Edge<EED>> {
        &mut self.edges
    }

    pub fn get_verts_mut(&mut self) -> &mut Arena<Vert<EVD>> {
        &mut self.verts
    }

    pub fn get_nodes_mut(&mut self) -> &mut Arena<Node<END>> {
        &mut self.nodes
    }

    pub fn get_face(&self, index: FaceIndex) -> Option<&Face<EFD>> {
        self.faces.get(index.0)
    }

    pub fn get_edge(&self, index: EdgeIndex) -> Option<&Edge<EED>> {
        self.edges.get(index.0)
    }

    pub fn get_vert(&self, index: VertIndex) -> Option<&Vert<EVD>> {
        self.verts.get(index.0)
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&Node<END>> {
        self.nodes.get(index.0)
    }

    pub fn get_face_mut(&mut self, index: FaceIndex) -> Option<&mut Face<EFD>> {
        self.faces.get_mut(index.0)
    }

    pub fn get_edge_mut(&mut self, index: EdgeIndex) -> Option<&mut Edge<EED>> {
        self.edges.get_mut(index.0)
    }

    pub fn get_vert_mut(&mut self, index: VertIndex) -> Option<&mut Vert<EVD>> {
        self.verts.get_mut(index.0)
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut Node<END>> {
        self.nodes.get_mut(index.0)
    }

    pub fn get_checked_verts_of_edge(
        &self,
        edge: &Edge<EED>,
        verts_swapped: bool,
    ) -> (&Vert<EVD>, &Vert<EVD>) {
        let verts = edge.get_verts().unwrap();

        if verts_swapped {
            (
                self.get_vert(verts.1).unwrap(),
                self.get_vert(verts.0).unwrap(),
            )
        } else {
            (
                self.get_vert(verts.0).unwrap(),
                self.get_vert(verts.1).unwrap(),
            )
        }
    }

    pub fn get_checked_nodes_of_edge(
        &self,
        edge: &Edge<EED>,
        nodes_swapped: bool,
    ) -> (&Node<END>, &Node<END>) {
        let (v1, v2) = self.get_checked_verts_of_edge(edge, nodes_swapped);

        (
            self.get_node(v1.node.unwrap()).unwrap(),
            self.get_node(v2.node.unwrap()).unwrap(),
        )
    }

    pub fn get_connecting_edge_indices(&self, n1: &Node<END>, n2: &Node<END>) -> Vec<EdgeIndex> {
        n1.get_verts()
            .iter()
            .flat_map(|v1_index| {
                n2.get_verts()
                    .iter()
                    .map(move |v2_index| (v1_index, v2_index))
            })
            .filter_map(|(v1_index, v2_index)| self.get_connecting_edge_index(*v1_index, *v2_index))
            .collect()
    }

    pub fn is_edge_loose(&self, edge: &Edge<EED>) -> bool {
        edge.is_loose()
    }

    pub fn is_edge_on_seam(&self, edge: &Edge<EED>) -> bool {
        edge.is_on_seam()
    }

    pub fn is_edge_on_boundary(&self, edge: &Edge<EED>) -> bool {
        let (n1, n2) = self.get_checked_nodes_of_edge(edge, false);
        let edge_indices = self.get_connecting_edge_indices(n1, n2);

        let num_faces = edge_indices.iter().try_fold(0, |acc, e_index| {
            let val = acc + self.get_edge(*e_index).unwrap().get_faces().len();
            if val > 1 {
                None
            } else {
                Some(val)
            }
        });
        num_faces.is_some()
    }

    pub fn is_edge_loose_or_on_seam_or_boundary(&self, edge: &Edge<EED>) -> bool {
        self.is_edge_loose(edge) || self.is_edge_on_seam(edge) || self.is_edge_on_boundary(edge)
    }

    pub fn is_edge_flippable(&self, edge: &Edge<EED>, across_seams: bool) -> bool {
        if across_seams {
            todo!("Need to handle across seams")
        }

        // ensure 2 faces only
        if edge.get_faces().len() != 2 {
            return false;
        }

        // ensure triangulation
        edge.get_faces()
            .iter()
            .try_for_each(|face_index| {
                (self.get_face(*face_index).unwrap().get_verts().len() == 3).then(|| ())
            })
            .is_some()
    }

    pub fn get_checked_other_vert_index(
        &self,
        edge_index: EdgeIndex,
        face_index: FaceIndex,
    ) -> VertIndex {
        let face = self.get_face(face_index).unwrap();

        assert_eq!(face.get_verts().len(), 3);

        let edge = self.get_edge(edge_index).unwrap();

        assert!(edge.get_faces().contains(&face_index));

        if !edge.has_vert(face.get_verts()[0]) {
            face.get_verts()[0]
        } else if !edge.has_vert(face.get_verts()[1]) {
            face.get_verts()[1]
        } else {
            assert!(!edge.has_vert(face.get_verts()[2]));
            face.get_verts()[2]
        }
    }

    /// Adds an empty Node and gives back mutable reference to it
    ///
    /// Use with caution
    fn add_empty_node(&mut self, pos: glm::DVec3) -> &mut Node<END> {
        let node_index = self
            .nodes
            .insert_with(|self_index| Node::new(NodeIndex(self_index), pos));
        &mut self.nodes[node_index]
    }

    /// Adds an empty Vert and gives back mutable reference to it
    ///
    /// Use with caution
    fn add_empty_vert(&mut self) -> &mut Vert<EVD> {
        let vert_index = self
            .verts
            .insert_with(|self_index| Vert::new(VertIndex(self_index)));
        &mut self.verts[vert_index]
    }

    /// Adds an empty Vert and gives index of it
    ///
    /// Use with caution
    fn add_empty_vert_index(&mut self) -> VertIndex {
        let vert_index = self
            .verts
            .insert_with(|self_index| Vert::new(VertIndex(self_index)));
        VertIndex(vert_index)
    }

    /// Adds an empty Edge and gives index of it
    ///
    /// Use with caution
    fn add_empty_edge_index(&mut self) -> EdgeIndex {
        let edge_index = self
            .edges
            .insert_with(|self_index| Edge::new(EdgeIndex(self_index)));
        EdgeIndex(edge_index)
    }

    /// Adds an empty Face and gives index of it
    ///
    /// Use with caution
    fn add_empty_face_index(&mut self) -> FaceIndex {
        let face_index = self
            .faces
            .insert_with(|self_index| Face::new(FaceIndex(self_index)));
        FaceIndex(face_index)
    }

    /// Gives the connecting edge index if there exists one
    pub fn get_connecting_edge_index(
        &self,
        vert_1_index: VertIndex,
        vert_2_index: VertIndex,
    ) -> Option<EdgeIndex> {
        for edge_index in &self.verts.get(vert_1_index.0)?.edges {
            let edge = self.edges.get(edge_index.0)?;
            if edge.has_vert(vert_2_index) {
                return Some(*edge_index);
            }
        }

        None
    }

    pub fn read(data: &MeshIO) -> Result<Self, MeshError> {
        let mut mesh = Mesh::new();

        if data.uvs.is_empty() || !data.face_has_uv {
            return Err(MeshError::NoUV);
        }

        // Create all the nodes
        for pos in &data.positions {
            mesh.add_empty_node(*pos);
        }

        // Create all the verts
        for uv in &data.uvs {
            let vert = mesh.add_empty_vert();
            vert.uv = Some(*uv);
        }

        // Work with the face indices that have been read to form the edges and faces
        for face_i in &data.face_indices {
            // Update verts and nodes
            for (pos_index, uv_index, normal_index) in face_i {
                let vert = mesh.verts.get_unknown_gen_mut(*uv_index).unwrap().0;
                let node = mesh.nodes.get_unknown_gen_mut(*pos_index).unwrap().0;

                // Update vert with node
                vert.node = Some(node.self_index);

                // Update node with vert
                node.verts.push(vert.self_index);
                // If MeshReader has found "vertex normal" information, store it in the Node
                if data.face_has_normal && !data.normals.is_empty() {
                    node.set_normal(data.normals[*normal_index]);
                }
            }

            let mut face_edges = Vec::new();
            let mut face_verts = Vec::new();

            // Update edges
            for ((_, vert_1_index, _), (_, vert_2_index, _)) in
                face_i.iter().circular_tuple_windows()
            {
                let vert_1_index = mesh.verts.get_unknown_gen_mut(*vert_1_index).unwrap().1;
                let vert_2_index = mesh.verts.get_unknown_gen_mut(*vert_2_index).unwrap().1;
                match mesh
                    .get_connecting_edge_index(VertIndex(vert_1_index), VertIndex(vert_2_index))
                {
                    Some(edge_index) => {
                        let edge = mesh.edges.get(edge_index.0).unwrap();
                        face_edges.push(edge.self_index);
                    }
                    None => {
                        let edge_index = mesh.add_empty_edge_index();
                        let edge = mesh.edges.get_mut(edge_index.0).unwrap();
                        // Update edge with vert
                        edge.verts = Some((VertIndex(vert_1_index), VertIndex(vert_2_index)));
                        // Update vert with edge
                        let vert_1 = mesh.verts.get_mut(vert_1_index).unwrap();
                        vert_1.edges.push(edge.self_index);
                        let vert_2 = mesh.verts.get_mut(vert_2_index).unwrap();
                        vert_2.edges.push(edge.self_index);
                        face_edges.push(edge.self_index);
                    }
                }

                face_verts.push(VertIndex(vert_1_index));
            }

            // Update faces
            {
                let face_index = mesh.add_empty_face_index();
                let face = mesh.faces.get_mut(face_index.0).unwrap();
                // Update face with verts
                face.verts = face_verts;

                // Update edges with face
                for edge_index in &face_edges {
                    let edge = mesh.edges.get_mut(edge_index.0).unwrap();
                    edge.faces.push(face.self_index);
                }
            }
        }

        // Any node without a vert gets a new vert without uv
        let mut loose_nodes = Vec::new();
        mesh.nodes
            .iter()
            .filter(|(_, node)| node.verts.is_empty())
            .for_each(|(_, node)| {
                loose_nodes.push(node.self_index);
            });
        for node_index in loose_nodes {
            let vert_index = mesh.add_empty_vert_index();
            let vert = mesh.verts.get_mut(vert_index.0).unwrap();
            let node = mesh.nodes.get_mut(node_index.0).unwrap();
            vert.node = Some(node.self_index);
            node.verts.push(vert.self_index);
        }

        // Add lines to the mesh
        for line in &data.line_indices {
            for (node_index_1, node_index_2) in line.iter().tuple_windows() {
                // Since lines don't store the UV information, we take
                // the nodes' first vert to create the edge
                let edge_index = mesh.add_empty_edge_index();
                let edge = mesh.edges.get_mut(edge_index.0).unwrap();

                let node_1 = mesh.nodes.get_unknown_gen(*node_index_1).unwrap().0;
                let node_2 = mesh.nodes.get_unknown_gen(*node_index_2).unwrap().0;

                let vert_1 = mesh.verts.get(node_1.verts[0].0).unwrap();
                let vert_2 = mesh.verts.get(node_2.verts[0].0).unwrap();

                // Update edge with verts
                edge.verts = Some((vert_1.self_index, vert_2.self_index));

                // Update verts with edge
                let vert_1 = mesh.verts.get_mut(node_1.verts[0].0).unwrap();
                vert_1.edges.push(edge.self_index);
                let vert_2 = mesh.verts.get_mut(node_2.verts[0].0).unwrap();
                vert_2.edges.push(edge.self_index);
            }
        }

        Ok(mesh)
    }

    pub fn read_from_file(path: &Path) -> Result<Self, MeshError> {
        let data = MeshIO::read(path)?;
        Self::read(&data)
    }

    pub fn apply_model_matrix(&mut self, model: &glm::DMat4) {
        // TODO(ish): need figure out exactly what parts (position,
        // normal, etc.) need this model matrix applied. As of right
        // now, only assuming position and normal
        self.get_nodes_mut().iter_mut().for_each(|(_, node)| {
            node.pos =
                glm::vec4_to_vec3(&(model * glm::vec4(node.pos[0], node.pos[1], node.pos[2], 1.0)));
            node.normal = node
                .normal
                .map(|normal| glm::vec4_to_vec3(&(model * glm::vec3_to_vec4(&normal))));
        });
    }

    pub fn unapply_model_matrix(&mut self, model: &glm::DMat4) {
        let inverse_model = model.try_inverse().unwrap();
        self.apply_model_matrix(&inverse_model);
    }

    /// Gets the nodes of the face in the same order as the verts of
    /// the face.
    ///
    /// If the vert doesn't exist or the vert doesn't store
    /// a node, that position will be None in the Vec
    pub fn get_nodes_of_face(&self, face: &Face<EFD>) -> Vec<Option<NodeIndex>> {
        face.get_verts()
            .iter()
            .map(|vert_index| self.get_vert(*vert_index).and_then(|vert| vert.node))
            .collect()
    }

    fn draw_smooth_color_3d_shader(
        &self,
        draw_data: &mut MeshDrawData,
    ) -> Result<(), MeshDrawError> {
        if self.faces.is_empty() {
            return Ok(());
        }

        let color = draw_data
            .color
            .ok_or(MeshDrawError::NoColorButSmoothColorShader)?;

        let imm = &mut draw_data.imm;

        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();

        smooth_color_3d_shader.use_shader();

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );
        let color_attr = format.add_attribute(
            "in_color\0".to_string(),
            GPUVertCompType::F32,
            4,
            GPUVertFetchMode::Float,
        );

        imm.begin_at_most(
            GPUPrimType::Tris,
            self.faces.len() * 10,
            smooth_color_3d_shader,
        );

        for (_, face) in &self.faces {
            let verts = &face.verts;
            let vert_1_index = verts[0];
            let vert_1 = self.verts.get(vert_1_index.0).unwrap();
            let node_1 = self.nodes.get(vert_1.node.unwrap().0).unwrap();
            for (vert_2_index, vert_3_index) in verts.iter().skip(1).tuple_windows() {
                let vert_2 = self.verts.get(vert_2_index.0).unwrap();
                let vert_3 = self.verts.get(vert_3_index.0).unwrap();

                let node_2 = self.nodes.get(vert_2.node.unwrap().0).unwrap();
                let node_3 = self.nodes.get(vert_3.node.unwrap().0).unwrap();

                imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
                let node_1_pos: glm::Vec3 = glm::convert(node_1.pos);
                imm.vertex_3f(pos_attr, node_1_pos[0], node_1_pos[1], node_1_pos[2]);

                imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
                let node_2_pos: glm::Vec3 = glm::convert(node_2.pos);
                imm.vertex_3f(pos_attr, node_2_pos[0], node_2_pos[1], node_2_pos[2]);

                imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
                let node_3_pos: glm::Vec3 = glm::convert(node_3.pos);
                imm.vertex_3f(pos_attr, node_3_pos[0], node_3_pos[1], node_3_pos[2]);
            }
        }

        imm.end();

        Ok(())
    }

    fn draw_directional_light_shader(
        &self,
        draw_data: &mut MeshDrawData,
    ) -> Result<(), MeshDrawError> {
        if self.faces.is_empty() {
            return Ok(());
        }

        let imm = &mut draw_data.imm;
        let directional_light_shader = shader::builtins::get_directional_light_shader()
            .as_ref()
            .unwrap();

        directional_light_shader.use_shader();

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );
        // let uv_attr = format.add_attribute(
        //     "in_uv\0".to_string(),
        //     GPUVertCompType::F32,
        //     2,
        //     GPUVertFetchMode::Float,
        // );
        let normal_attr = format.add_attribute(
            "in_normal\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );

        imm.begin_at_most(
            GPUPrimType::Tris,
            self.faces.len() * 10,
            directional_light_shader,
        );

        for (_, face) in &self.faces {
            let verts = &face.verts;
            let vert_1_index = verts[0];
            let vert_1 = self.verts.get(vert_1_index.0).unwrap();
            let node_1 = self.nodes.get(vert_1.node.unwrap().0).unwrap();
            for (vert_2_index, vert_3_index) in verts.iter().skip(1).tuple_windows() {
                let vert_2 = self.verts.get(vert_2_index.0).unwrap();
                let vert_3 = self.verts.get(vert_3_index.0).unwrap();

                let node_2 = self.nodes.get(vert_2.node.unwrap().0).unwrap();
                let node_3 = self.nodes.get(vert_3.node.unwrap().0).unwrap();

                let node_1_normal: glm::Vec3 = glm::convert(node_1.normal.unwrap());
                imm.attr_3f(
                    normal_attr,
                    node_1_normal[0],
                    node_1_normal[1],
                    node_1_normal[2],
                );
                // imm.attr_2f(uv_attr, 0.0, 0.0);
                let node_1_pos: glm::Vec3 = glm::convert(node_1.pos);
                imm.vertex_3f(pos_attr, node_1_pos[0], node_1_pos[1], node_1_pos[2]);

                let node_2_normal: glm::Vec3 = glm::convert(node_2.normal.unwrap());
                imm.attr_3f(
                    normal_attr,
                    node_2_normal[0],
                    node_2_normal[1],
                    node_2_normal[2],
                );
                // imm.attr_2f(uv_attr, 0.0, 0.0);
                let node_2_pos: glm::Vec3 = glm::convert(node_2.pos);
                imm.vertex_3f(pos_attr, node_2_pos[0], node_2_pos[1], node_2_pos[2]);

                let node_3_normal: glm::Vec3 = glm::convert(node_3.normal.unwrap());
                imm.attr_3f(
                    normal_attr,
                    node_3_normal[0],
                    node_3_normal[1],
                    node_3_normal[2],
                );
                // imm.attr_2f(uv_attr, 0.0, 0.0);
                let node_3_pos: glm::Vec3 = glm::convert(node_3.pos);
                imm.vertex_3f(pos_attr, node_3_pos[0], node_3_pos[1], node_3_pos[2]);
            }
        }

        imm.end();

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MeshDrawError {
    GenerateGLMeshFirst,
    ErrorWhileDrawing,
    NoColorButSmoothColorShader,
}

pub enum MeshUseShader {
    DirectionalLight,
    SmoothColor3D,
}

pub struct MeshDrawData<'a> {
    imm: &'a mut GPUImmediate,
    use_shader: MeshUseShader,
    color: Option<glm::Vec4>,
}

impl<'a> MeshDrawData<'a> {
    pub fn new(
        imm: &'a mut GPUImmediate,
        use_shader: MeshUseShader,
        color: Option<glm::Vec4>,
    ) -> Self {
        MeshDrawData {
            imm,
            use_shader,
            color,
        }
    }
}

impl std::fmt::Display for MeshDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshDrawError::GenerateGLMeshFirst => {
                write!(f, "Generate GLMesh before calling draw()")
            }
            MeshDrawError::ErrorWhileDrawing => {
                write!(f, "Error while drawing Mesh")
            }
            MeshDrawError::NoColorButSmoothColorShader => write!(
                f,
                "No color provided in draw data but asking to use smooth color 3D shader"
            ),
        }
    }
}

impl std::error::Error for MeshDrawError {}

impl From<()> for MeshDrawError {
    fn from(_err: ()) -> MeshDrawError {
        MeshDrawError::ErrorWhileDrawing
    }
}

impl<END, EVD, EED, EFD> Drawable<MeshDrawData<'_>, MeshDrawError> for Mesh<END, EVD, EED, EFD> {
    fn draw(&self, draw_data: &mut MeshDrawData) -> Result<(), MeshDrawError> {
        match draw_data.use_shader {
            MeshUseShader::DirectionalLight => self.draw_directional_light_shader(draw_data),
            MeshUseShader::SmoothColor3D => self.draw_smooth_color_3d_shader(draw_data),
        }
    }

    fn draw_wireframe(&self, draw_data: &mut MeshDrawData) -> Result<(), MeshDrawError> {
        let imm = &mut draw_data.imm;

        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();

        smooth_color_3d_shader.use_shader();

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            3,
            GPUVertFetchMode::Float,
        );
        let color_attr = format.add_attribute(
            "in_color\0".to_string(),
            GPUVertCompType::F32,
            4,
            GPUVertFetchMode::Float,
        );

        imm.begin(
            GPUPrimType::Lines,
            self.edges.len() * 2,
            smooth_color_3d_shader,
        );

        for (_, edge) in &self.edges {
            let (vert_1_index, vert_2_index) = edge.get_verts().unwrap();
            let vert_1 = self.verts.get(vert_1_index.0).unwrap();
            let vert_2 = self.verts.get(vert_2_index.0).unwrap();
            let node_1_index = vert_1.node.unwrap();
            let node_2_index = vert_2.node.unwrap();
            let node_1 = self.nodes.get(node_1_index.0).unwrap();
            let node_2 = self.nodes.get(node_2_index.0).unwrap();
            let node_1_pos: glm::Vec3 = glm::convert(node_1.pos);
            let node_2_pos: glm::Vec3 = glm::convert(node_2.pos);

            imm.attr_4f(color_attr, 0.8, 0.8, 0.8, 1.0);
            imm.vertex_3f(pos_attr, node_1_pos[0], node_1_pos[1], node_1_pos[2]);
            imm.attr_4f(color_attr, 1.0, 1.0, 1.0, 1.0);
            imm.vertex_3f(pos_attr, node_2_pos[0], node_2_pos[1], node_2_pos[2]);
        }

        imm.end();

        Ok(())
    }
}

impl<T> Face<T> {
    pub fn new(self_index: FaceIndex) -> Face<T> {
        Face {
            self_index,
            normal: None,
            extra_data: None,

            verts: Vec::new(),
        }
    }

    pub fn get_verts(&self) -> &AdjacentVerts {
        &self.verts
    }

    /// # Safety
    ///
    /// Use this only if you know what you are doing. It is
    /// possible to completely destroy the Mesh structure by using
    /// this
    pub unsafe fn get_verts_mut(&mut self) -> &mut AdjacentVerts {
        &mut self.verts
    }
}

impl<T> Edge<T> {
    pub fn new(self_index: EdgeIndex) -> Edge<T> {
        Edge {
            self_index,
            extra_data: None,

            verts: None,
            faces: Vec::new(),
        }
    }

    pub fn get_self_index(&self) -> EdgeIndex {
        self.self_index
    }

    pub fn get_verts(&self) -> &Option<(VertIndex, VertIndex)> {
        &self.verts
    }

    pub fn get_faces(&self) -> &IncidentFaces {
        &self.faces
    }

    /// Checks if self has the vert specified via VertIndex
    pub fn has_vert(&self, vert_index: VertIndex) -> bool {
        match self.verts {
            Some((v1_index, v2_index)) => {
                if v1_index == vert_index {
                    true
                } else {
                    v2_index == vert_index
                }
            }
            None => false,
        }
    }

    /// Returns the other vert's index given that a valid index (an
    /// index part of self.verts) otherwise returns None
    pub fn get_other_vert_index(&self, vert_index: VertIndex) -> Option<VertIndex> {
        match self.verts {
            Some((v1_index, v2_index)) => {
                if v1_index == vert_index {
                    Some(v2_index)
                } else if v2_index == vert_index {
                    Some(v1_index)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Swaps the ordering of the vert indices in self.verts if it exists
    pub fn swap_verts(&mut self) {
        if let Some((v1_index, v2_index)) = self.verts {
            self.verts = Some((v2_index, v1_index));
        }
    }

    pub fn is_loose(&self) -> bool {
        self.faces.is_empty()
    }

    pub fn is_on_seam(&self) -> bool {
        self.get_faces().len() == 1
    }

    /// # Safety
    ///
    /// Use this only if you know what you are doing. It is
    /// possible to completely destroy the Mesh structure by using
    /// this
    pub unsafe fn get_verts_mut(&mut self) -> &mut Option<(VertIndex, VertIndex)> {
        &mut self.verts
    }

    /// # Safety
    ///
    /// Use this only if you know what you are doing. It is
    /// possible to completely destroy the Mesh structure by using
    /// this
    pub unsafe fn get_faces_mut(&mut self) -> &mut IncidentFaces {
        &mut self.faces
    }
}

impl<T> Vert<T> {
    pub fn new(self_index: VertIndex) -> Vert<T> {
        Vert {
            self_index,
            uv: None,
            extra_data: None,

            node: None,
            edges: Vec::new(),
        }
    }

    pub fn get_node(&self) -> &Option<NodeIndex> {
        &self.node
    }

    pub fn get_edges(&self) -> &IncidentEdges {
        &self.edges
    }

    /// # Safety
    ///
    /// Use this only if you know what you are doing. It is
    /// possible to completely destroy the Mesh structure by using
    /// this
    pub unsafe fn get_edges_mut(&mut self) -> &mut IncidentEdges {
        &mut self.edges
    }

    /// # Safety
    ///
    /// Use this only if you know what you are doing. It is
    /// possible to completely destroy the Mesh structure by using
    /// this
    pub unsafe fn get_node_mut(&mut self) -> &mut Option<NodeIndex> {
        &mut self.node
    }
}

impl<T> Node<T> {
    pub fn new(self_index: NodeIndex, pos: glm::DVec3) -> Node<T> {
        Node {
            self_index,
            pos,
            normal: None,
            extra_data: None,

            verts: Vec::new(),
        }
    }

    pub fn set_normal(&mut self, normal: glm::DVec3) {
        self.normal = Some(normal);
    }

    pub fn get_verts(&self) -> &IncidentVerts {
        &self.verts
    }

    /// # Safety
    ///
    /// Use this only if you know what you are doing. It is
    /// possible to completely destroy the Mesh structure by using
    /// this
    pub unsafe fn get_verts_mut(&mut self) -> &mut IncidentVerts {
        &mut self.verts
    }
}

pub mod simple {
    pub type Node = super::Node<()>;
    pub type Vert = super::Vert<()>;
    pub type Edge = super::Edge<()>;
    pub type Face = super::Face<()>;
    pub type Mesh = super::Mesh<(), (), (), ()>;
}

fn _add_as_set<T>(vec: &mut Vec<T>, val: T)
where
    T: PartialEq,
{
    if vec.contains(&val) {
        return;
    }
    vec.push(val);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_read_test() {
        // TODO(ish): add more comprehensive relation tests
        let mesh = simple::Mesh::read_from_file(&Path::new("tests/obj_test_01.obj")).unwrap();
        assert_eq!(mesh.faces.len(), 2);
        for (_, face) in &mesh.faces {
            assert_eq!(face.verts.len(), 3);
        }
        assert_eq!(mesh.edges.len(), 7);
        for (_, edge) in &mesh.edges {
            assert!(edge.verts.is_some());
        }
        assert_eq!(mesh.verts.len(), 7);
        for (_, vert) in &mesh.verts {
            let len = vert.edges.len();
            assert!(len == 1 || len == 2 || len == 3);
        }
        assert_eq!(mesh.nodes.len(), 5);
        for (_, node) in &mesh.nodes {
            let len = node.verts.len();
            assert!(len == 0 || len == 1 || len == 2);
        }
    }

    #[test]
    fn mesh_no_uv() {
        let res = simple::Mesh::read_from_file(&Path::new("tests/obj_test_05_square_no_uv.obj"));
        if let Err(err) = res {
            match err {
                MeshError::NoUV => {}
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }
    }
}
