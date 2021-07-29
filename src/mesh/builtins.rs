use lazy_static::lazy_static;

use crate::meshreader::MeshReader;

use super::simple::Mesh;

macro_rules! load_builtin_mesh {
    ( $name:ident ; $get_str_name:ident ; $get_name:ident ; $static_name:ident ; $location:tt ) => {
        lazy_static! {
            static ref $static_name: Mesh = {
                let file = $get_str_name();
                let lines: Vec<&str> = file.split('\n').collect();
                let reader = MeshReader::from_lines(&lines).unwrap();
                Mesh::read(&reader).unwrap()
            };
        }

        pub fn $get_str_name() -> &'static str {
            include_str!($location)
        }
        pub fn $get_name() -> &'static Mesh {
            &$static_name
        }
    };
}

load_builtin_mesh!(cube_subd_00; get_cube_subd_00_obj_str; get_cube_subd_00; CUBE_SUBD_00; "../../models/cube_subd_00.obj");
load_builtin_mesh!(cube_subd_00_triangulated; get_cube_subd_00_triangulated_obj_str; get_cube_subd_00_triangulated; CUBE_SUBD_00_TRIANGULATED; "../../models/cube_subd_00_triangulated.obj");

load_builtin_mesh!(ico_sphere_subd_00; get_ico_sphere_subd_00_obj_str; get_ico_sphere_subd_00; ICO_SPHERE_SUBD_00; "../../models/ico_sphere_subd_00.obj");
load_builtin_mesh!(ico_sphere_subd_01; get_ico_sphere_subd_01_obj_str; get_ico_sphere_subd_01; ICO_SPHERE_SUBD_01; "../../models/ico_sphere_subd_01.obj");
load_builtin_mesh!(ico_sphere_subd_02; get_ico_sphere_subd_02_obj_str; get_ico_sphere_subd_02; ICO_SPHERE_SUBD_02; "../../models/ico_sphere_subd_02.obj");

load_builtin_mesh!(monkey_subd_00; get_monkey_subd_00_obj_str; get_monkey_subd_00; MONKEY_SUBD_00; "../../models/monkey_subd_00.obj");
load_builtin_mesh!(monkey_subd_00_triangulated; get_monkey_subd_00_triangulated_obj_str; get_monkey_subd_00_triangulated; MONKEY_SUBD_00_TRIANGULATED; "../../models/monkey_subd_00_triangulated.obj");

load_builtin_mesh!(monkey_subd_01; get_monkey_subd_01_obj_str; get_monkey_subd_01; MONKEY_SUBD_01; "../../models/monkey_subd_01.obj");
load_builtin_mesh!(monkey_subd_01_triangulated; get_monkey_subd_01_triangulated_obj_str; get_monkey_subd_01_triangulated; MONKEY_SUBD_01_TRIANGULATED; "../../models/monkey_subd_01_triangulated.obj");

load_builtin_mesh!(plane_subd_00; get_plane_subd_00_obj_str; get_plane_subd_00; PLANE_SUBD_00; "../../models/plane_subd_00.obj");
load_builtin_mesh!(plane_subd_00_triangulated; get_plane_subd_00_triangulated_obj_str; get_plane_subd_00_triangulated; PLANE_SUBD_00_TRIANGULATED; "../../models/plane_subd_00_triangulated.obj");
