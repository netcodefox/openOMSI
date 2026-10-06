fn main() {
    use std::path::Path;
    let base = Path::new(r\"e:/SteamLibrary/steamapps/common/OMSI 2/Vehicles/AC O530/Model/ibox\");
    let names = [
        \"ibox_brightness.o3d\", \"ibox_ibox_taste_d10.o3d\", \"ibox_ibox_taste_d11.o3d\",
        \"ibox_ibox_taste_d12.o3d\", \"ibox_ibox_taste_d9.o3d\", \"ibox_body.o3d\",
        \"ibox_text_1.o3d\", \"ibox_text_2.o3d\", \"ibox_text_3.o3d\", \"ibox_karte.o3d\",
    ];
    for name in names {
        let path = base.join(name);
        match omsi_o3d::load_mesh(&path) {
            Ok(m) => {
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for v in &m.vertices {
                    let p = v.position;
                    min[0]=min[0].min(p.x); min[1]=min[1].min(p.y); min[2]=min[2].min(p.z);
                    max[0]=max[0].max(p.x); max[1]=max[1].max(p.y); max[2]=max[2].max(p.z);
                }
                let c = [
                    (min[0]+max[0])*0.5, (min[1]+max[1])*0.5, (min[2]+max[2])*0.5,
                ];
                println!("{name}: tris={} verts={} min=({:.4},{:.4},{:.4}) max=({:.4},{:.4},{:.4}) center=({:.4},{:.4},{:.4}) xform={:?}",
                    m.triangles.len(), m.vertices.len(), min[0],min[1],min[2], max[0],max[1],max[2], c[0],c[1],c[2], m.has_transform);
            }
            Err(e) => println!("{name}: ERR {e}"),
        }
    }
}
