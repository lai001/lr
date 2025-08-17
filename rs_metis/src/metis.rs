use crate::{
    bindings::{
        gk_free, graph_t, libmetis__CreateGraph, libmetis__FreeGraph, libmetis__InitGraph,
        libmetis__imalloc, libmetis__ismalloc, libmetis__rsmalloc, mctype_et_METIS_CTYPE_SHEM,
        miptype_et_METIS_IPTYPE_METISRB, mobjtype_et_METIS_OBJTYPE_CUT,
        moptions_et_METIS_OPTION_CONTIG, moptions_et_METIS_OPTION_CTYPE,
        moptions_et_METIS_OPTION_DBGLVL, moptions_et_METIS_OPTION_DROPEDGES,
        moptions_et_METIS_OPTION_IPTYPE, moptions_et_METIS_OPTION_MINCONN,
        moptions_et_METIS_OPTION_NCUTS, moptions_et_METIS_OPTION_NIPARTS,
        moptions_et_METIS_OPTION_NITER, moptions_et_METIS_OPTION_NO2HOP,
        moptions_et_METIS_OPTION_OBJTYPE, moptions_et_METIS_OPTION_ONDISK,
        moptions_et_METIS_OPTION_RTYPE, moptions_et_METIS_OPTION_SEED,
        moptions_et_METIS_OPTION_UFACTOR, mptype_et_METIS_PTYPE_KWAY, mptype_et_METIS_PTYPE_RB,
        mrtype_et_METIS_RTYPE_FM, mrtype_et_METIS_RTYPE_GREEDY, params_t, rstatus_et_METIS_OK,
        METIS_PartGraphKway, METIS_PartGraphRecursive, METIS_SetDefaultOptions,
        KMETIS_DEFAULT_UFACTOR, MCPMETIS_DEFAULT_UFACTOR, METIS_NOPTIONS, PMETIS_DEFAULT_UFACTOR,
    },
    edge::Edge,
    graph::{Graph, GraphVertexIndex, MeshVertexIndex, TriangleGraph},
    vertex_position::VertexPosition,
};
use std::{
    collections::HashSet,
    num::NonZero,
    ops::{Deref, DerefMut, Range},
    process::Command,
    sync::Arc,
};

fn loop_range_next(value: usize, range: Range<usize>) -> usize {
    range.start + (value + 1) % (range.end - range.start)
}

fn loop_range_triangle_next(value: usize) -> usize {
    let start = value / 3 * 3;
    let end = start + 3;
    loop_range_next(value, start..end)
}

pub struct Metis {}

impl Metis {
    fn find_other_two_vertex_indices(at: usize, indices: &[u32]) -> [u32; 2] {
        let next_at = loop_range_triangle_next(at);
        let first = indices[next_at];
        let next_at = loop_range_triangle_next(next_at);
        let second = indices[next_at];
        [first, second]
    }

    fn make_adjoin_graph_vertex_indices(
        indices: &[u32],
        vertices: &[glam::Vec3],
    ) -> Vec<HashSet<GraphVertexIndex>> {
        let mut adjoin_indices: Vec<HashSet<GraphVertexIndex>> = Vec::new();
        adjoin_indices.resize(vertices.len(), HashSet::new());

        for (at, vertex_index) in indices.iter().enumerate() {
            let other_two_vertex_indices = Self::find_other_two_vertex_indices(at, indices);
            adjoin_indices[*vertex_index as usize].extend(other_two_vertex_indices);
        }
        adjoin_indices
    }

    fn make_edges(adjoin_graph_vertex_indices: &[HashSet<GraphVertexIndex>]) -> HashSet<Edge> {
        let mut edges: HashSet<Edge> = HashSet::new();
        for (graph_vertex_index, adjoin_indices) in adjoin_graph_vertex_indices.iter().enumerate() {
            for adjoin_vertex_index in adjoin_indices.clone() {
                let edge = Edge::new(graph_vertex_index as u32, adjoin_vertex_index);
                edges.insert(edge);
            }
        }
        edges
    }

    fn make_graph_vertex_associated_indices(
        indices: &[u32],
        vertices: &[glam::Vec3],
    ) -> Vec<HashSet<usize>> {
        let mut graph_vertex_associated_indices: Vec<HashSet<usize>> = Vec::new();
        graph_vertex_associated_indices.resize(vertices.len(), HashSet::new());
        for (at, vertex_index) in indices.iter().enumerate() {
            graph_vertex_associated_indices[*vertex_index as usize].insert(at);
        }
        graph_vertex_associated_indices
    }

    fn to_graph(indices: &[u32], vertices: &[glam::Vec3]) -> Graph {
        debug_assert_eq!(indices.len() % 3, 0);
        let adjoin_graph_vertex_indices = Self::make_adjoin_graph_vertex_indices(indices, vertices);
        let edges = Self::make_edges(&adjoin_graph_vertex_indices);
        let graph_vertex_associated_indices =
            Self::make_graph_vertex_associated_indices(indices, vertices);

        Graph {
            adjoin_indices: adjoin_graph_vertex_indices,
            graph_vertex_associated_indices,
            edges,
        }
    }

    // fn build_mesh_clusters(
    //     indices: &[u32],
    //     graph: &Graph,
    //     partitions: &Vec<Vec<GraphVertexIndex>>,
    // ) -> Vec<Vec<MeshVertexIndex>> {
    //     let mut cluster_indices: Vec<Vec<MeshVertexIndex>> = Vec::new();
    //     for partition in partitions {
    //         let mut triangles: HashSet<(usize, usize, usize)> = HashSet::new();
    //         for graph_vertex_index in partition {
    //             let associated_indices =
    //                 &graph.graph_vertex_associated_indices[*graph_vertex_index as usize];
    //             for index in associated_indices {
    //                 let triangle = Self::fill_indices_triangle(*index);
    //                 triangles.insert(triangle);
    //             }
    //         }
    //         let sub_indices: Vec<GraphVertexIndex> = triangles
    //             .iter()
    //             .flat_map(|triangle| {
    //                 [
    //                     indices[triangle.0],
    //                     indices[triangle.1],
    //                     indices[triangle.2],
    //                 ]
    //             })
    //             .collect();
    //         cluster_indices.push(sub_indices);
    //     }
    //     cluster_indices
    // }

    fn build_mesh_clusters(
        graph: &Graph,
        partitions: &Vec<Vec<GraphVertexIndex>>,
    ) -> Vec<Vec<usize>> {
        let mut cluster_indices: Vec<Vec<usize>> = Vec::new();
        for partition in partitions {
            let mut triangles: HashSet<usize> = HashSet::new();
            for graph_vertex_index in partition {
                let associated_indices =
                    &graph.graph_vertex_associated_indices[*graph_vertex_index as usize];
                for index in associated_indices {
                    let triangle = *index / 3 * 3;
                    triangles.insert(triangle);
                }
            }
            cluster_indices.push(triangles.iter().map(|x| *x).collect::<Vec<usize>>());
        }
        cluster_indices
    }

    // fn fill_indices_triangle(index: usize) -> (usize, usize, usize) {
    //     let start = index / 3 * 3;
    //     (start, start + 1, start + 2)
    // }

    pub fn partition(
        indices: &[u32],
        vertices: &[glam::Vec3],
        num_parts: u32,
        gpmetis_program_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<Vec<Vec<usize>>> {
        let output_path = std::path::Path::new("./t.graph").to_path_buf();
        let output_path = rs_foundation::absolute_path(output_path)
            .map_err(|err| crate::error::Error::IO(err, None))?;
        let graph = Self::to_graph(indices, vertices);
        Self::write_graph(&graph, output_path.clone())?;
        let partition = Self::internal_partition(gpmetis_program_path, output_path, num_parts)?;
        // let _ = std::fs::remove_file(output_path);

        let mut partition_ret: Vec<Vec<GraphVertexIndex>> = vec![vec![]; num_parts as usize];

        for (graph_vertex_index, which_part) in partition.iter().enumerate() {
            let value = partition_ret
                .get_mut(*which_part as usize)
                .expect("Should not be null");
            value.push(graph_vertex_index as GraphVertexIndex);
        }

        Ok(Self::build_mesh_clusters(&graph, &partition_ret))
    }

    fn internal_partition(
        gpmetis_program_path: impl AsRef<std::path::Path>,
        graph_file_path: impl AsRef<std::path::Path>,
        num_parts: u32,
    ) -> crate::error::Result<Vec<u32>> {
        let mut cmd = Command::new(gpmetis_program_path.as_ref());
        cmd.args([
            graph_file_path
                .as_ref()
                .to_str()
                .ok_or(crate::error::Error::Other(Some(format!(""))))?,
            &num_parts.to_string(),
        ]);
        let output = cmd.output();
        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(crate::error::Error::Other(Some(format!(
                        "{}",
                        String::from_utf8(output.stderr)
                            .map_err(|err| crate::error::Error::FromUtf8Error(err))?
                    ))));
                }
            }
            Err(err) => {
                return Err(crate::error::Error::IO(err, None));
            }
        }

        let file_name = graph_file_path
            .as_ref()
            .file_name()
            .map(|x| x.to_str().map(|x| format!("{x}.part.{}", num_parts)))
            .flatten()
            .ok_or(crate::error::Error::Other(Some(format!("No parent"))))?;

        let graph_partition_file_path = graph_file_path.as_ref().with_file_name(file_name);
        let file = std::fs::File::open(graph_partition_file_path.clone()).map_err(|err| {
            crate::error::Error::IO(err, Some(format!("{:?}", graph_partition_file_path)))
        })?;
        let reader = std::io::BufReader::new(file);
        let mut partition: Vec<u32> = Vec::new();
        for line in std::io::BufRead::lines(reader) {
            let which_part: u32 = line
                .map_err(|err| crate::error::Error::IO(err, None))?
                .trim()
                .parse()
                .map_err(|err| crate::error::Error::ParseIntError(err))?;
            partition.push(which_part);
        }
        // let _ = std::fs::remove_file(graph_partition_file_path);
        Ok(partition)
    }

    fn write_graph(
        graph: &Graph,
        output_path: impl AsRef<std::path::Path>,
    ) -> crate::error::Result<()> {
        let mut content = String::new();
        content.push_str(&format!(
            "{} {}\n",
            graph.get_num_vertices(),
            graph.get_num_edges()
        ));
        for indices in &graph.adjoin_indices {
            let line: String = indices
                .iter()
                .map(|x| (x + 1).to_string())
                .collect::<Vec<String>>()
                .join(" ");
            content.push_str(&format!("{}\n", line));
        }
        std::fs::create_dir_all(
            output_path
                .as_ref()
                .parent()
                .ok_or(crate::error::Error::Other(Some(format!("No parent"))))?,
        )
        .map_err(|err| crate::error::Error::IO(err, None))?;
        if output_path.as_ref().exists() {
            std::fs::remove_file(output_path.as_ref())
                .map_err(|err| crate::error::Error::IO(err, None))?;
        }
        std::fs::write(output_path, content).map_err(|err| crate::error::Error::IO(err, None))
    }

    pub fn partition_from_graph(
        triangle_graph: &TriangleGraph,
        num_parts: NonZero<u32>,
    ) -> crate::error::Result<Vec<TriangleGraph>> {
        let _ = tracy_client::span!();
        let partition = Self::internal_partition_memory(&triangle_graph, num_parts.get())?;
        let mut partition_result: Vec<Vec<usize>> =
            vec![Vec::with_capacity(partition.len()); num_parts.get() as usize];
        for (graph_vertex_index, which_part) in partition.iter().enumerate() {
            (&mut partition_result[*which_part as usize]).push(graph_vertex_index);
        }
        if partition_result.len() != num_parts.get() as usize {
            return Err(crate::error::Error::Other(None));
        }
        let mut reuslts = Vec::with_capacity(partition_result.len());
        for selection in &mut partition_result {
            selection.shrink_to_fit();
            let sub_graph = TriangleGraph::from_cache(
                triangle_graph.get_triangles(),
                triangle_graph.get_adjoin_triangles(),
                selection,
            );
            reuslts.push(sub_graph);
        }
        Ok(reuslts)
    }

    pub fn partition_from_indexed_vertices(
        indices: &[u32],
        vertices: &[VertexPosition],
        num_parts: u32,
    ) -> crate::error::Result<Vec<Vec<MeshVertexIndex>>> {
        let _ = tracy_client::span!();
        let triangle_graph = TriangleGraph::from_indexed_vertices(indices, vertices);

        // let partition = Self::internal_partition(gpmetis_program_path, &output_path, num_parts)?;
        let partition = Self::internal_partition_memory(&triangle_graph, num_parts)?;

        let mut partition_result: Vec<Vec<GraphVertexIndex>> = vec![vec![]; num_parts as usize];

        for (graph_vertex_index, which_part) in partition.iter().enumerate() {
            let value = partition_result
                .get_mut(*which_part as usize)
                .expect("Should not be null");
            value.push(graph_vertex_index as GraphVertexIndex);
        }
        Ok(Self::build_mesh_clusters2(
            &triangle_graph,
            &partition_result,
        ))
    }

    pub fn parallel_partition_from_indexed_vertices(
        indices: &[u32],
        vertices: Arc<Vec<VertexPosition>>,
        num_parts: u32,
    ) -> crate::error::Result<Vec<Vec<MeshVertexIndex>>> {
        let _ = tracy_client::span!();

        let triangle_graph =
            TriangleGraph::parallel_from_indexed_vertices(indices, vertices.clone());
        unsafe { Self::metis_graph_from_triangle_graph(&triangle_graph).unwrap() };

        // let partition = Self::internal_partition(gpmetis_program_path, &output_path, num_parts)?;
        let partition = Self::internal_partition_memory(&triangle_graph, num_parts)?;

        let mut partition_result: Vec<Vec<GraphVertexIndex>> = vec![vec![]; num_parts as usize];

        for (graph_vertex_index, which_part) in partition.iter().enumerate() {
            let value = partition_result
                .get_mut(*which_part as usize)
                .expect("Should not be null");
            value.push(graph_vertex_index as GraphVertexIndex);
        }
        Ok(Self::build_mesh_clusters2(
            &triangle_graph,
            &partition_result,
        ))
    }

    fn build_mesh_clusters2(
        graph: &TriangleGraph,
        partitions: &Vec<Vec<GraphVertexIndex>>,
    ) -> Vec<Vec<MeshVertexIndex>> {
        let mut cluster_indices: Vec<Vec<MeshVertexIndex>> = Vec::new();
        for partition in partitions {
            let mut triangles: Vec<MeshVertexIndex> = Vec::with_capacity(partition.len() * 3);
            for triangle_index in partition {
                let triangle = &graph.get_triangles()[*triangle_index as usize];
                triangles.append(&mut triangle.get_indices().to_vec());
            }
            cluster_indices.push(triangles);
        }
        cluster_indices
    }

    fn internal_partition_memory(
        triangle_graph: &TriangleGraph,
        num_parts: u32,
    ) -> crate::error::Result<Vec<u32>> {
        let _ = tracy_client::span!();
        unsafe {
            let mut graph = Self::metis_graph_from_triangle_graph(triangle_graph)?;
            let mut options: Vec<i32> = vec![0; METIS_NOPTIONS as usize];
            METIS_SetDefaultOptions(options.as_mut_ptr());

            let mut params = params_t {
                ptype: mptype_et_METIS_PTYPE_KWAY as crate::bindings::idx_t,
                objtype: mobjtype_et_METIS_OBJTYPE_CUT as crate::bindings::idx_t,
                ctype: mctype_et_METIS_CTYPE_SHEM as crate::bindings::idx_t,
                iptype: -1,
                rtype: -1,
                no2hop: 0,
                minconn: 0,
                contig: 0,
                ondisk: 0,
                dropedges: 0,
                nooutput: 0,
                balance: 0,
                ncuts: 1,
                niter: 10,
                niparts: -1,
                gtype: 0,
                ncommon: 0,
                seed: -1,
                dbglvl: 0,
                nparts: 1,
                nseps: 0,
                ufactor: -1,
                pfactor: 0,
                compress: 0,
                ccorder: 0,
                filename: std::ptr::null_mut(),
                outfile: std::ptr::null_mut(),
                xyzfile: std::ptr::null_mut(),
                tpwgtsfile: std::ptr::null_mut(),
                ubvecstr: std::ptr::null_mut(),
                wgtflag: 3,
                numflag: 0,
                tpwgts: std::ptr::null_mut(),
                ubvec: std::ptr::null_mut(),
                iotimer: 0.0,
                parttimer: 0.0,
                reporttimer: 0.0,
                maxmemory: 0,
            };

            let msg = "ReadTPwgts: tpwgts";
            #[cfg(target_arch = "x86_64")]
            let msg_raw = msg.as_ptr() as *mut i8;
            #[cfg(not(target_arch = "x86_64"))]
            let msg_raw = msg.as_ptr() as *mut u8;

            params.tpwgts =
                libmetis__rsmalloc((params.nparts * graph.ncon) as usize, -1.0, msg_raw);
            params.nparts = num_parts as i32;
            params.iptype = miptype_et_METIS_IPTYPE_METISRB as crate::bindings::idx_t;

            if params.ptype == mptype_et_METIS_PTYPE_RB as crate::bindings::idx_t {
                params.rtype = mrtype_et_METIS_RTYPE_FM as crate::bindings::idx_t;
            } else if params.ptype == mptype_et_METIS_PTYPE_KWAY as crate::bindings::idx_t {
                params.iptype = if params.iptype != -1 {
                    params.iptype
                } else {
                    miptype_et_METIS_IPTYPE_METISRB as crate::bindings::idx_t
                };
                params.rtype = mrtype_et_METIS_RTYPE_GREEDY as crate::bindings::idx_t;
            }

            if params.ufactor == -1 {
                if params.ptype == mptype_et_METIS_PTYPE_KWAY as crate::bindings::idx_t {
                    params.ufactor = KMETIS_DEFAULT_UFACTOR as crate::bindings::idx_t;
                } else if graph.ncon == 1 {
                    params.ufactor = PMETIS_DEFAULT_UFACTOR as crate::bindings::idx_t;
                } else {
                    params.ufactor = MCPMETIS_DEFAULT_UFACTOR as crate::bindings::idx_t;
                }
            }

            if params.tpwgtsfile == std::ptr::null_mut() {
                for i in 0..params.nparts {
                    for j in 0..graph.ncon {
                        *params.tpwgts.wrapping_add((i * graph.ncon + j) as usize) =
                            1.0 / params.nparts as f32;
                    }
                }
            }

            options[moptions_et_METIS_OPTION_OBJTYPE as usize] = params.objtype;
            options[moptions_et_METIS_OPTION_CTYPE as usize] = params.ctype;
            options[moptions_et_METIS_OPTION_IPTYPE as usize] = params.iptype;
            options[moptions_et_METIS_OPTION_RTYPE as usize] = params.rtype;
            options[moptions_et_METIS_OPTION_NO2HOP as usize] = params.no2hop;
            options[moptions_et_METIS_OPTION_ONDISK as usize] = params.ondisk;
            options[moptions_et_METIS_OPTION_DROPEDGES as usize] = params.dropedges;
            options[moptions_et_METIS_OPTION_MINCONN as usize] = params.minconn;
            options[moptions_et_METIS_OPTION_CONTIG as usize] = params.contig;
            options[moptions_et_METIS_OPTION_SEED as usize] = params.seed;
            options[moptions_et_METIS_OPTION_NIPARTS as usize] = params.niparts;
            options[moptions_et_METIS_OPTION_NITER as usize] = params.niter;
            options[moptions_et_METIS_OPTION_NCUTS as usize] = params.ncuts;
            options[moptions_et_METIS_OPTION_UFACTOR as usize] = params.ufactor;
            options[moptions_et_METIS_OPTION_DBGLVL as usize] = params.dbglvl;

            let mut objval: i32 = 0;
            let mut part: Vec<i32> = vec![0; graph.nvtxs as usize];

            if params.ptype == mptype_et_METIS_PTYPE_RB as crate::bindings::idx_t {
                let status = METIS_PartGraphRecursive(
                    &mut graph.nvtxs,
                    &mut graph.ncon,
                    graph.xadj,
                    graph.adjncy,
                    graph.vwgt,
                    graph.vsize,
                    graph.adjwgt,
                    &mut params.nparts,
                    params.tpwgts,
                    params.ubvec,
                    options.as_mut_ptr(),
                    &mut objval,
                    part.as_mut_ptr(),
                );
                if status != rstatus_et_METIS_OK {
                    return Err(crate::error::Error::Other(Some(format!(
                        "Metis returned with an error."
                    ))));
                }
            } else if params.ptype == mptype_et_METIS_PTYPE_KWAY as crate::bindings::idx_t {
                let status = METIS_PartGraphKway(
                    &mut graph.nvtxs,
                    &mut graph.ncon,
                    graph.xadj,
                    graph.adjncy,
                    graph.vwgt,
                    graph.vsize,
                    graph.adjwgt,
                    &mut params.nparts,
                    params.tpwgts,
                    params.ubvec,
                    options.as_mut_ptr(),
                    &mut objval,
                    part.as_mut_ptr(),
                );
                if status != rstatus_et_METIS_OK {
                    return Err(crate::error::Error::Other(Some(format!(
                        "Metis returned with an error."
                    ))));
                }
            } else {
                return Err(crate::error::Error::Other(Some(format!("Not support"))));
            }

            gk_free(
                (&mut params.tpwgts) as *mut *mut f32 as *mut *mut std::ffi::c_void,
                std::ptr::null_mut() as *mut *mut std::ffi::c_void,
            );
            Ok(part.iter().map(|x| *x as u32).collect())
        }
    }

    unsafe fn metis_graph_from_triangle_graph(
        triangle_graph: &TriangleGraph,
    ) -> crate::error::Result<Box<MetisGraph>> {
        let mut graph = MetisGraph::new();

        let fmt = 0;
        if fmt > 111 {
            return Err(crate::error::Error::Other(Some(format!(
                "Cannot read this type of file format [fmt={}]!",
                fmt
            ))));
        }
        let fmtstr = format!("{:03}", fmt % 1000);
        let _readvs = fmtstr.chars().skip(0).next().unwrap() == '1';
        let _readvw = fmtstr.chars().skip(1).next().unwrap() == '1';
        let readew = fmtstr.chars().skip(2).next().unwrap() == '1';

        let ncon = 1;
        graph.nvtxs = triangle_graph.get_graph_vertices_len() as i32;
        graph.nedges = triangle_graph.get_graph_edges_len() as i32;
        if graph.nvtxs <= 0 || graph.nedges <= 0 {
            return Err(crate::error::Error::Other(Some(format!(
                "The supplied nvtxs:{} and nedges:{} must be positive.",
                graph.nvtxs, graph.nedges
            ))));
        }
        graph.nedges *= 2;
        graph.ncon = ncon;

        graph.xadj = libmetis__ismalloc(
            graph.nvtxs as usize + 1,
            0,
            "ReadGraph: xadj".as_ptr() as *mut ::std::os::raw::c_char,
        );

        graph.adjncy = libmetis__imalloc(
            graph.nedges as usize,
            "ReadGraph: adjncy".as_ptr() as *mut ::std::os::raw::c_char,
        );

        graph.vwgt = libmetis__ismalloc(
            ncon as usize * graph.nvtxs as usize,
            1,
            "ReadGraph: vwgt".as_ptr() as *mut ::std::os::raw::c_char,
        );

        if readew {
            graph.adjwgt = libmetis__imalloc(
                graph.nedges as usize,
                "ReadGraph: adjwgt".as_ptr() as *mut ::std::os::raw::c_char,
            );
        } else {
            graph.adjwgt = std::ptr::null_mut();
        }

        graph.vsize = libmetis__ismalloc(
            graph.nvtxs as usize,
            1,
            "ReadGraph: vsize".as_ptr() as *mut ::std::os::raw::c_char,
        );

        let mut k: usize = 0;
        let adjoin_triangles = triangle_graph.get_adjoin_triangles();
        for (i, adjoin_triangle) in adjoin_triangles.iter().enumerate() {
            for edge in adjoin_triangle.clone() {
                let edge = edge + 1;
                if edge < 1 || edge > graph.nvtxs as u32 {
                    return Err(crate::error::Error::Other(Some(format!(
                        "Edge {} for vertex {} is out of bounds",
                        edge,
                        i + 1
                    ))));
                }

                if k == graph.nedges as usize {
                    return Err(crate::error::Error::Other(Some(format!(
                        "There are more edges in the file than the {} specified.",
                        graph.nedges / 2
                    ))));
                }

                *graph.adjncy.wrapping_add(k) = (edge - 1) as i32;
                k = k + 1;
            }
            *graph.xadj.wrapping_add(i + 1) = k as i32;
        }

        if k != graph.nedges as usize {
            let err_msg = format!(
                r"You specified that the graph contained {} edges. However, I only found {} edges in the graph.
Please specify the correct number of edges in the graph.",
                graph.nedges / 2,
                k / 2
            );
            return Err(crate::error::Error::Other(Some(err_msg)));
        }

        Ok(graph)
    }
}

#[derive(Debug)]
struct MetisGraph {
    inner: *mut graph_t,
}

impl Deref for MetisGraph {
    type Target = graph_t;

    fn deref(&self) -> &Self::Target {
        assert_ne!(self.inner, std::ptr::null_mut());
        unsafe { self.inner.as_ref().unwrap() }
    }
}

impl DerefMut for MetisGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        assert_ne!(self.inner, std::ptr::null_mut());
        unsafe { self.inner.as_mut().unwrap() }
    }
}

impl MetisGraph {
    fn new() -> Box<MetisGraph> {
        let inner = unsafe { libmetis__CreateGraph() };
        assert_ne!(inner, std::ptr::null_mut());
        unsafe { libmetis__InitGraph(inner) };
        Box::new(MetisGraph { inner })
    }
}

impl Drop for MetisGraph {
    fn drop(&mut self) {
        unsafe { libmetis__FreeGraph(&mut self.inner) };
    }
}
