# modified form https://blender.stackexchange.com/questions/80460/slice-up-terrain-mesh-into-chunks
# made it also rename to decent names to make chunking easier
import bpy, bmesh
from bpy import context
from  mathutils import Vector
# bounding box helper methods
def bbox(ob):
    return (Vector(b) for b in ob.bound_box)

def bbox_center(ob):
    return sum(bbox(ob), Vector()) / 8

def bbox_axes(ob):
    bb = list(bbox(ob))
    return tuple(bb[i] for i in (0, 4, 3, 1))

def slice(bm, start, end, segments):
    if segments == 1:
        return
    def geom(bm):
        return bm.verts[:] + bm.edges[:] + bm.faces[:]
    planes = [start.lerp(end, f / segments) for f in range(1, segments)]
    #p0 = start
    plane_no = (end - start).normalized() 
    while(planes): 
        p0 = planes.pop(0)                 
        ret = bmesh.ops.bisect_plane(bm, 
                geom=geom(bm),
                plane_co=p0, 
                plane_no=plane_no)
        bmesh.ops.split_edges(bm, 
                edges=[e for e in ret['geom_cut'] 
                if isinstance(e, bmesh.types.BMEdge)])


bm = bmesh.new()
ob = context.object
me = ob.data
bm.from_mesh(me)
name = ob.name


o, x, y, z = bbox_axes(ob)        

x_segments = 8
y_segments = 1
z_segments = 8

slice(bm, o, x, x_segments)
slice(bm, o, y, y_segments)
slice(bm, o, z, z_segments)    
bm.to_mesh(me)

bpy.ops.object.mode_set(mode='EDIT')
foo = bpy.ops.mesh.separate(type='LOOSE')
print(foo)
bpy.ops.object.mode_set()
bpy.ops.object.select_all( action = 'SELECT' )
bpy.ops.object.origin_set( type = 'ORIGIN_GEOMETRY' )


scaler = 8
offset = 4
for o in context.scene.objects:
    if o.name.startswith(name):
        bm = bmesh.new()
        bm.from_mesh(o.data)
        bmesh.ops.triangulate(bm, faces=bm.faces)
        bm.to_mesh(o.data)
        
        x,y,z = o.location
        x = int((x + 0.5) * x_segments)
        y = int((-y + 0.5) * z_segments)
        o.name = f"{x}x{y}"
        o.data.name = o.name
