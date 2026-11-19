"""Helpers compartilhados pelos scripts Blender (paleta 64x64, partes, export, previa).

Convencoes: escala em metros; Blender +Z (cima) vira +Y no glTF; frente do personagem =
Blender +Y, que vira -Z no glTF (convencao Bevy).
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector


def out_path():
    argv = sys.argv
    if "--" not in argv or len(argv) <= argv.index("--") + 1:
        raise SystemExit("uso: blender --background --python <script>.py -- <saida.glb>")
    return os.path.abspath(argv[argv.index("--") + 1])


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def hex_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4))


class Palette:
    """Grade 8x8 de blocos 8x8 px. Cada cor nomeada ocupa um bloco."""

    def __init__(self, colors):
        self.names = list(colors.keys())
        assert len(self.names) <= 64
        self.colors = colors
        px = [0.0] * (64 * 64 * 4)
        for y in range(64):
            for x in range(64):
                i = (y // 8) * 8 + (x // 8)
                if i < len(self.names):
                    r, g, b = hex_rgb(colors[self.names[i]])
                else:
                    r, g, b = 1.0, 0.0, 1.0
                o = (y * 64 + x) * 4
                px[o:o + 4] = [r, g, b, 1.0]
        self.image = bpy.data.images.new("palette", 64, 64, alpha=False)
        self.image.pixels = px
        self.image.pack()
        self.material = self._make_material()

    def uv(self, name):
        i = self.names.index(name)
        return Vector(((i % 8 + 0.5) / 8.0, (i // 8 + 0.5) / 8.0))

    def _make_material(self):
        m = bpy.data.materials.new("palette")
        m.use_nodes = True
        nt = m.node_tree
        bsdf = nt.nodes["Principled BSDF"]
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = self.image
        tex.interpolation = "Closest"
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        bsdf.inputs["Roughness"].default_value = 1.0
        return m


class Part:
    """Acumula geometria num bmesh; cada primitiva recebe um bloco de cor."""

    def __init__(self, pal):
        self.pal = pal
        self.bm = bmesh.new()
        self.uvl = self.bm.loops.layers.uv.new("UVMap")

    # -- utilidades -------------------------------------------------
    def _end(self, verts, color, center, rot, scale=(1, 1, 1)):
        M = (Matrix.Translation(center) @ Euler(rot).to_matrix().to_4x4()
             @ Matrix.Diagonal((*scale, 1.0)))
        for v in verts:
            v.co = M @ v.co
        uv = self.pal.uv(color)
        faces = {f for v in verts for f in v.link_faces}
        for f in faces:
            for l in f.loops:
                l[self.uvl].uv = uv

    def transform_all(self, M):
        for v in self.bm.verts:
            v.co = M @ v.co

    # -- primitivas -------------------------------------------------
    def box(self, center, size, color, top=(1, 1), rot=(0, 0, 0)):
        """Caixa; `top` escala x,y dos vertices de cima (afunilamento)."""
        vs = bmesh.ops.create_cube(self.bm, size=1.0)["verts"]
        for v in vs:
            if v.co.z > 0:
                v.co.x *= top[0]
                v.co.y *= top[1]
        self._end(vs, color, center, rot, size)

    def cyl(self, center, r_bot, r_top, h, color, seg=8, rot=(0, 0, 0), scale=(1, 1)):
        vs = bmesh.ops.create_cone(self.bm, cap_ends=True, cap_tris=False, segments=seg,
                                   radius1=r_bot, radius2=r_top, depth=h)["verts"]
        self._end(vs, color, center, rot, (scale[0], scale[1], 1))

    def sphere(self, center, radii, color, u=8, v=6, rot=(0, 0, 0)):
        vs = bmesh.ops.create_uvsphere(self.bm, u_segments=u, v_segments=v, radius=1.0)["verts"]
        self._end(vs, color, center, rot, radii)

    def loft(self, profile, levels, colors, color_caps, center=(0, 0, 0)):
        """Extrusao com aneis. profile: [(x,y)] anti-horario; levels: [(z, sx, sy)];
        colors: cor por aresta do perfil. Tampas em cima/baixo."""
        rings = []
        for z, sx, sy in levels:
            rings.append([self.bm.verts.new((x * sx, y * sy, z)) for x, y in profile])
        n = len(profile)
        for k in range(len(rings) - 1):
            for i in range(n):
                j = (i + 1) % n
                f = self.bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
                uv = self.pal.uv(colors[i])
                for l in f.loops:
                    l[self.uvl].uv = uv
        for ring, rev in ((rings[0], True), (rings[-1], False)):
            f = self.bm.faces.new(list(reversed(ring)) if rev else ring)
            uv = self.pal.uv(color_caps)
            for l in f.loops:
                l[self.uvl].uv = uv
        for ring in rings:
            for v in ring:
                v.co += Vector(center)

    # -- saida ------------------------------------------------------
    def to_object(self, name, location=(0, 0, 0), parent=None):
        mesh = bpy.data.meshes.new(name)
        self.bm.normal_update()
        self.bm.to_mesh(mesh)
        self.bm.free()
        for p in mesh.polygons:
            p.use_smooth = False
        mesh.materials.append(self.pal.material)
        obj = bpy.data.objects.new(name, mesh)
        bpy.context.scene.collection.objects.link(obj)
        obj.parent = parent
        obj.location = location
        return obj


def tri_count(objs):
    dg = bpy.context.evaluated_depsgraph_get()
    total = 0
    for o in objs:
        if o.type == "MESH":
            m = o.evaluated_get(dg).to_mesh()
            m.calc_loop_triangles()
            total += len(m.loop_triangles)
    return total


def export_glb(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=path, export_format="GLB", export_yup=True, export_apply=True,
        export_normals=True, export_texcoords=True, export_materials="EXPORT",
        export_cameras=False, export_lights=False, use_selection=False)


def render_preview(png_path, view_dir, ortho_pad=1.15, size=(640, 800), up_hint=(0, 0, 1)):
    """Workbench com textura; camera ortografica olhando de `view_dir` (Blender)."""
    scn = bpy.context.scene
    meshes = [o for o in scn.objects if o.type == "MESH"]
    pts = [o.matrix_world @ Vector(c) for o in meshes for c in o.bound_box]
    lo = Vector((min(p.x for p in pts), min(p.y for p in pts), min(p.z for p in pts)))
    hi = Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts)))
    center = (lo + hi) / 2
    d = Vector(view_dir).normalized()
    cam_data = bpy.data.cameras.new("cam")
    cam_data.type = "ORTHO"
    cam = bpy.data.objects.new("cam", cam_data)
    scn.collection.objects.link(cam)
    cam.location = center + d * 20
    cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
    bpy.context.view_layer.update()
    # escala ortografica: projeta caixa na camera
    inv = cam.matrix_world.inverted()
    pc = [inv @ p for p in pts]
    w = max(p.x for p in pc) - min(p.x for p in pc)
    h = max(p.y for p in pc) - min(p.y for p in pc)
    aspect = size[0] / size[1]
    cam_data.ortho_scale = (max(w, h * aspect) if aspect >= 1 else max(h, w / aspect)) * ortho_pad
    shift = Vector(((max(p.x for p in pc) + min(p.x for p in pc)) / 2,
                    (max(p.y for p in pc) + min(p.y for p in pc)) / 2, 0))
    cam.location += cam.matrix_world.to_3x3() @ shift
    bpy.context.view_layer.update()
    scn.camera = cam
    scn.render.engine = "BLENDER_WORKBENCH"
    scn.render.resolution_x, scn.render.resolution_y = size
    scn.render.image_settings.file_format = "PNG"
    scn.render.filepath = png_path
    sh = scn.display.shading
    sh.light = "STUDIO"
    sh.color_type = "TEXTURE"
    sh.show_object_outline = True
    scn.world = bpy.data.worlds.new("w")
    scn.world.color = (0.35, 0.4, 0.45)
    scn.view_settings.view_transform = "Standard"
    bpy.ops.render.render(write_still=True)


def finish(name, objs, glb, view_dir, size=(640, 800)):
    print(f"[{name}] triangulos: {tri_count(objs)}")
    export_glb(glb)
    render_preview(os.path.splitext(glb)[0] + ".png", view_dir, size=size)
    print(f"[{name}] exportado: {glb}")


def build_sword(part, scale=1.0, flip=False, lite=False):
    """Espada longa em +Z com origem no centro do punho. flip: lamina aponta para -Z."""
    P = part
    if lite:
        P.cyl((0, 0, 0), 0.021, 0.021, 0.20, "leather", seg=6, scale=(1, 0.85))
        P.box((0, 0, -0.125), (0.05, 0.05, 0.05), "gold")
        P.box((0, 0, 0.12), (0.24, 0.03, 0.024), "gold")
        _blade(P)
        _finish_sword(P, scale, flip)
        return
    # punho (couro) com anéis
    P.cyl((0, 0, 0), 0.021, 0.021, 0.20, "leather", seg=10, scale=(1, 0.85))
    for z in (-0.06, -0.02, 0.02, 0.06):
        P.cyl((0, 0, z), 0.0245, 0.0245, 0.012, "leather_dark", seg=10, scale=(1, 0.85))
    # pomo
    P.cyl((0, 0, -0.112), 0.03, 0.034, 0.024, "gold", seg=10)
    P.sphere((0, 0, -0.14), (0.034, 0.034, 0.034), "gold", u=10, v=6)
    # guarda cruzada com pontas levemente voltadas pra lamina
    P.box((0, 0, 0.112), (0.05, 0.04, 0.024), "gold")
    P.box((0, 0, 0.122), (0.24, 0.026, 0.022), "gold", top=(0.92, 1))
    for sx in (-1, 1):
        P.box((sx * 0.125, 0, 0.134), (0.03, 0.026, 0.04), "gold", top=(0.6, 0.8))
        P.sphere((sx * 0.122, 0, 0.122), (0.017, 0.017, 0.017), "gold_dark", u=8, v=5)
    _blade(P)
    # sulco central (fuller) nas duas faces
    for sy in (-1, 1):
        P.box((0, sy * 0.0058, 0.40), (0.012, 0.002, 0.46), "steel_dark")
    _finish_sword(P, scale, flip)


def _blade(P):
    prof = [(-0.5, 0), (-0.32, 0.5), (0.32, 0.5), (0.5, 0), (0.32, -0.5), (-0.32, -0.5)]
    cols = ["steel_edge", "steel", "steel_edge", "steel_edge", "steel", "steel_edge"]
    W, T = 0.052, 0.011
    P.loft(prof, [(0.134, W * 0.95, T), (0.17, W, T), (0.60, W * 0.96, T * 0.9),
                  (0.74, W * 0.78, T * 0.7), (0.81, W * 0.4, T * 0.4), (0.845, 0.004, 0.003)],
           cols, "steel_edge")


def _finish_sword(P, scale, flip):
    if scale != 1.0:
        P.transform_all(Matrix.Scale(scale, 4))
    if flip:
        P.transform_all(Matrix.Rotation(math.pi, 4, "X"))


SWORD_COLORS = {
    "steel": "#c9d1d9", "steel_edge": "#eef2f6", "steel_dark": "#8e99a6",
    "leather": "#5b3a22", "leather_dark": "#3a2414", "gold": "#c9a227", "gold_dark": "#8a6d12",
}
