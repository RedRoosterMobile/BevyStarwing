"""Run inside Blender using MCP. Builds additive scene content; never deletes existing objects."""
import bpy
import math
from pathlib import Path
from mathutils import Vector, Matrix

ROOT = Path('/Users/thomasranker/code/bevy/starwing')
OUT = ROOT / 'assets/characters/panther'
OUT.mkdir(parents=True, exist_ok=True)
assert 'Panther_Character' not in bpy.data.collections, 'Character already exists; do not duplicate it.'
EXPRESSIONS = ['smile', 'celebrate', 'fear_scream_hurt', 'talk']
collection = bpy.data.collections.new('Panther_Character')
bpy.context.scene.collection.children.link(collection)
parts = []

def material(name, color, roughness=0.8):
    m = bpy.data.materials.new('Panther_' + name)
    m.diffuse_color = (*color, 1)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Roughness'].default_value = roughness
    return m

fur = material('Midnight', (0.035, 0.055, 0.085))
cheek = material('Cheeks', (0.07, 0.105, 0.145))
muzzle = material('Muzzle', (0.19, 0.25, 0.29))
black = material('Nose_and_Mouth', (0.009, 0.013, 0.019))
ear = material('Inner_Ear', (0.34, 0.20, 0.24))
white = material('Ivory', (0.87, 0.88, 0.72))
iris = material('Amber', (0.78, 0.52, 0.055))
tongue = material('Tongue', (0.55, 0.12, 0.20))
collar = material('Flight_Collar', (0.085, 0.31, 0.30))

def finish(obj, name, mat):
    obj.name = name
    for c in list(obj.users_collection):
        c.objects.unlink(obj)
    collection.objects.link(obj)
    obj.data.materials.append(mat)
    for p in obj.data.polygons:
        p.use_smooth = True
    parts.append(obj)
    return obj

def ellipsoid(name, center, scale, mat, variants=None, angle=0, cone=False):
    if cone:
        bpy.ops.mesh.primitive_cone_add(vertices=20, radius1=1, radius2=0.05, depth=2)
    else:
        bpy.ops.mesh.primitive_uv_sphere_add(segments=24, ring_count=16)
    obj = finish(bpy.context.object, name, mat)
    local = [v.co.copy() for v in obj.data.vertices]
    def coordinates(c, s, a):
        rot = Matrix.Rotation(a, 3, 'Y')
        return [Vector(c) + rot @ Vector((v.x*s[0], v.y*s[1], v.z*s[2])) for v in local]
    for v, co in zip(obj.data.vertices, coordinates(center, scale, angle)):
        v.co = co
    obj.shape_key_add(name='Basis')
    for expression in EXPRESSIONS:
        key = obj.shape_key_add(name=expression)
        c, s, a = (variants or {}).get(expression, (center, scale, angle))
        for v, co in zip(key.data, coordinates(c, s, a)):
            v.co = co
    return obj

ellipsoid('Head', (0, 0, 0.25), (0.94, 0.76, 1.03), fur)
ellipsoid('Neck', (0, 0.12, -1.00), (0.47, 0.43, 0.53), fur)
ellipsoid('Flight_Collar', (0, 0.11, -1.32), (0.63, 0.49, 0.18), collar)
ellipsoid('Collar_Badge', (0, -0.385, -1.32), (0.10, 0.028, 0.065), white)
for side in [-1, 1]:
    x = side * 0.77
    ellipsoid('Ear', (x, 0.015, 1.12), (0.285, 0.21, 0.47), fur,
              {'fear_scream_hurt': ((side*0.89, 0.05, 0.99), (0.30, 0.20, 0.37), side*0.65)}, angle=side*0.25)
    ellipsoid('Inner_Ear', (x, -0.17, 1.18), (0.175, 0.045, 0.29), ear,
              {'fear_scream_hurt': ((side*0.92, -0.13, 1.02), (0.19, 0.035, 0.23), side*0.65)}, angle=side*0.25)
    ellipsoid('Cheek', (side*0.67, -0.38, -0.15), (0.36, 0.46, 0.48), cheek)
    ellipsoid('Muzzle', (side*0.285, -0.97, -0.16), (0.36, 0.30, 0.255), muzzle,
              {'smile': ((side*0.31, -0.97, -0.13), (0.37, 0.30, 0.24), 0),
               'celebrate': ((side*0.32, -0.97, -0.12), (0.37, 0.30, 0.235), 0)})
    # All layers of each eye deform together, keeping the pupil on the eye surface.
    for name, y, s, mat in [('Eye', -0.815, (0.295, 0.215, 0.26), white),
                             ('Iris', -1.018, (0.125, 0.036, 0.17), iris),
                             ('Pupil', -1.053, (0.046, 0.018, 0.135), black),
                             ('Eye_Glint', -1.073, (0.029, 0.012, 0.040), white)]:
        xx = side*0.43 + (-0.035 if name == 'Eye_Glint' else 0)
        zz = 0.47 + (0.065 if name == 'Eye_Glint' else 0)
        variants = {}
        for expression, factor, rise in [('smile', 0.72, 0.015), ('celebrate', 0.44, 0.035), ('fear_scream_hurt', 1.18, 0.005), ('talk', 0.92, 0)]:
            variants[expression] = ((xx, y, 0.47+(zz-0.47)*factor+rise), (s[0], s[1], s[2]*factor), 0)
        ellipsoid(name, (xx, y, zz), s, mat, variants)
    ellipsoid('Expressive_Brow', (side*0.44, -0.915, 0.79), (0.31, 0.12, 0.075), fur,
              {'smile': ((side*0.44, -0.915, 0.81), (0.30, 0.12, 0.075), side*0.08),
               'celebrate': ((side*0.44, -0.915, 0.85), (0.30, 0.12, 0.075), side*0.20),
               'fear_scream_hurt': ((side*0.44, -0.915, 0.90), (0.30, 0.12, 0.075), side*0.38),
               'talk': ((side*0.44, -0.915, 0.82), (0.30, 0.12, 0.075), -side*0.12)}, angle=-side*0.10)
    for dx, dz in [(0, 0), (0.12, -0.025), (0.065, -0.10)]:
        ellipsoid('Whisker_Dot', (side*(0.27+dx), -1.264, -0.13+dz), (0.025, 0.014, 0.025), black)

ellipsoid('Nose', (0, -1.267, 0.055), (0.22, 0.14, 0.115), black)
ellipsoid('Nose_Highlight', (-0.06, -1.397, 0.084), (0.055, 0.008, 0.018), muzzle)
ellipsoid('Chin', (0, -0.83, -0.64), (0.46, 0.32, 0.25), muzzle,
          {'smile': ((0, -0.83, -0.68), (0.47, 0.32, 0.25), 0),
           'celebrate': ((0, -0.83, -0.91), (0.46, 0.32, 0.25), 0),
           'fear_scream_hurt': ((0, -0.83, -1.03), (0.40, 0.32, 0.25), 0),
           'talk': ((0, -0.83, -0.83), (0.45, 0.32, 0.25), 0)})
# Same mouth topology for all targets: dark inset and a soft lip perimeter.
mouth_states = {'Basis': (0.39, 0.019, -0.43, 0.045), 'smile': (0.47, 0.075, -0.46, 0.12),
                'celebrate': (0.46, 0.255, -0.57, 0.07), 'fear_scream_hurt': (0.34, 0.36, -0.61, -0.02),
                'talk': (0.39, 0.195, -0.53, 0.035)}
def mouth_coords(state, lip):
    w, h, z, curve = mouth_states[state]
    coords = []
    if not lip:
        coords.append((0, -1.282, z))
        for i in range(64):
            t = i*math.tau/64
            coords.append((w*math.cos(t), -1.282, z+h*math.sin(t)+curve*math.cos(t)**2))
    else:
        for i in range(64):
            t = i*math.tau/64
            for j in range(8):
                p = j*math.tau/8
                coords.append(((w+0.024*math.cos(p))*math.cos(t), -1.29-0.024*math.sin(p),
                               z+(h+0.024*math.cos(p))*math.sin(t)+curve*math.cos(t)**2))
    return coords
for lip, name, mat in [(False, 'Mouth_Interior', black), (True, 'Lip', cheek)]:
    faces = []
    if lip:
        for i in range(64):
            for j in range(8):
                faces.append((i*8+j, ((i+1)%64)*8+j, ((i+1)%64)*8+(j+1)%8, i*8+(j+1)%8))
    else:
        faces = [(0, 1+i, 1+(i+1)%64) for i in range(64)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(mouth_coords('Basis', lip), [], faces)
    obj = bpy.data.objects.new(name, mesh)
    collection.objects.link(obj)
    finish(obj, name, mat)
    obj.shape_key_add(name='Basis')
    for state in EXPRESSIONS:
        key = obj.shape_key_add(name=state)
        for v, co in zip(key.data, mouth_coords(state, lip)):
            v.co = co
for side in [-1, 1]:
    variants = {}
    for state in EXPRESSIONS:
        w,h,z,curve = mouth_states[state]
        x = side*w*0.56
        visible = state != 'smile'
        variants[state] = ((x, -1.305, z+h*0.83+curve*0.31-0.068),
                           (0.055, 0.025, 0.11) if visible else (0.001, 0.001, 0.001), math.pi)
    ellipsoid('Upper_Fang', (side*0.22, -1.305, -0.43), (0.001, 0.001, 0.001), white, variants, angle=math.pi, cone=True)
ellipsoid('Tongue', (0, -1.304, -0.44), (0.001, 0.001, 0.001), tongue,
          {state: ((0, -1.304, z-h*0.68), (w*0.46, 0.016, h*0.22), 0)
           for state, (w,h,z,curve) in mouth_states.items() if state in EXPRESSIONS})

bpy.ops.object.select_all(action='DESELECT')
for obj in parts:
    obj.select_set(True)
bpy.context.view_layer.objects.active = parts[0]
bpy.ops.object.join()
head = bpy.context.object
head.name = 'Panther_Head'
head.data.name = 'Panther_Head_Mesh'
assert [k.name for k in head.data.shape_keys.key_blocks] == ['Basis']+EXPRESSIONS
head['expressions'] = 'neutral = all weights zero; smile; celebrate; fear_scream_hurt; talk'
head['front'] = 'Blender -Y / glTF +Z; up +Y in glTF'
keys = head.data.shape_keys.key_blocks
# A short looping talking clip, so consumers may animate or drive weights directly.
for frame, value in [(1, 0), (5, 0.8), (9, 0.2), (13, 1), (17, 0.15), (21, 0.65), (25, 0)]:
    keys['talk'].value = value
    keys['talk'].keyframe_insert(data_path='value', frame=frame)
head.data.shape_keys.animation_data.action.name = 'Talk'
scene = bpy.context.scene
scene.render.fps = 24
scene.frame_set(1)
bpy.ops.export_scene.gltf(filepath=str(OUT/'panther_morphs.glb'), export_format='GLB', use_selection=True,
                          export_animations=True, export_animation_mode='ACTIONS', export_morph=True,
                          export_morph_normal=True, export_extras=True, export_texcoords=False)
# Baked pose GLBs work without a morph-animation integration.
head.data.shape_keys.animation_data_clear()
for state in ['neutral']+EXPRESSIONS:
    for key in list(keys)[1:]:
        key.value = float(key.name == state)
    bpy.context.view_layer.update()
    evaluated = head.evaluated_get(bpy.context.evaluated_depsgraph_get())
    mesh = bpy.data.meshes.new_from_object(evaluated)
    pose = bpy.data.objects.new('Panther_'+state, mesh)
    collection.objects.link(pose)
    head.select_set(False)
    pose.select_set(True)
    bpy.context.view_layer.objects.active = pose
    bpy.ops.export_scene.gltf(filepath=str(OUT/('panther_'+state+'.glb')), export_format='GLB', use_selection=True,
                              export_animations=False, export_texcoords=False)
    bpy.data.objects.remove(pose, do_unlink=True)
    bpy.data.meshes.remove(mesh)
    head.select_set(True)
    bpy.context.view_layer.objects.active = head
for key in list(keys)[1:]:
    key.value = 0
# Retain the editable Talk action in the source file.
action = bpy.data.actions.get('Talk')
if action:
    head.data.shape_keys.animation_data_create()
    head.data.shape_keys.animation_data.action = action
# Keep existing scene objects untouched; presentation uses a separate scene.
preview = bpy.data.scenes.new('Panther_Presentation')
preview.collection.children.link(collection)
bpy.context.window.scene = preview
preview.render.engine = 'CYCLES'
preview.cycles.samples = 24
preview.render.resolution_x = 640
preview.render.resolution_y = 640
preview.render.resolution_percentage = 100
preview.world = bpy.data.worlds.new('Panther_Studio')
preview.world.use_nodes = True
preview.world.node_tree.nodes['Background'].inputs[0].default_value = (0.025, 0.035, 0.055, 1)
preview.world.node_tree.nodes['Background'].inputs[1].default_value = 0.5
preview.view_settings.view_transform = 'AgX'
cam_data = bpy.data.cameras.new('Panther_Preview_Camera')
cam = bpy.data.objects.new('Panther_Preview_Camera', cam_data)
preview.collection.objects.link(cam)
cam.location = (0.12, -7, 0.48)
cam.rotation_euler = (Vector((0, 0, 0.08))-cam.location).to_track_quat('-Z','Y').to_euler()
cam_data.type = 'ORTHO'
cam_data.ortho_scale = 3.7
preview.camera = cam
for name, position, energy, color, size in [('Key', (-3,-4,4), 550, (0.78,0.88,1), 4),
                                          ('Fill', (3,-3,1), 350, (1,0.78,0.49), 3),
                                          ('Rim', (2,2,3), 700, (0.25,0.70,1), 3)]:
    data = bpy.data.lights.new('Panther_'+name, 'AREA')
    data.energy, data.color, data.shape, data.size = energy, color, 'DISK', size
    obj = bpy.data.objects.new('Panther_'+name, data)
    preview.collection.objects.link(obj)
    obj.location = position
    obj.rotation_euler = (Vector((0,0,0.2))-obj.location).to_track_quat('-Z','Y').to_euler()
preview.frame_set(1)
preview.render.image_settings.file_format = 'PNG'
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/panther_character.blend'))
result = {'source': bpy.data.filepath, 'exports': [str(p) for p in sorted(OUT.glob('*.glb'))],
          'vertices': len(head.data.vertices), 'shape_keys': [k.name for k in keys]}
