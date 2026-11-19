import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *

clear_scene()
pal = Palette({
    "wood": "#8a5f36", "wood_dark": "#5d3d22", "wood_light": "#a97c4c",
    "straw": "#d8bd62", "straw_dark": "#a98c3a", "sack": "#b59b6e", "sack_dark": "#7f6a45",
    "rope": "#6b4f2a", "target": "#a83232", "metal": "#7b828c",
})
p = Part(pal)
# base em cruz + poste
p.box((0, 0, 0.04), (0.70, 0.14, 0.08), "wood_dark")
p.box((0, 0, 0.04), (0.14, 0.70, 0.08), "wood_dark")
p.cyl((0, 0, 0.09), 0.10, 0.10, 0.02, "wood_dark", seg=8)
p.cyl((0, 0, 0.9), 0.055, 0.055, 1.62, "wood", seg=8)
# corpo de saco (barril de palha) amarrado com corda
p.cyl((0, 0, 1.12), 0.17, 0.20, 0.36, "sack", seg=8)
p.cyl((0, 0, 1.40), 0.20, 0.17, 0.20, "sack", seg=8)
p.cyl((0, 0, 1.12), 0.205, 0.205, 0.035, "rope", seg=8)
p.cyl((0, 0, 1.42), 0.205, 0.205, 0.035, "rope", seg=8)
p.cyl((0, 0, 0.93), 0.12, 0.15, 0.10, "sack_dark", seg=8)
# alvo pintado no peito (frente = +Y Blender = -Z glTF)
p.box((0, 0.193, 1.27), (0.16, 0.02, 0.16), "target")
# cabeca de saco com tufos de palha
p.sphere((0, 0, 1.64), (0.15, 0.15, 0.16), "sack", u=8, v=6)
p.cyl((0, 0, 1.55), 0.07, 0.09, 0.05, "rope", seg=8)
for a in range(0, 360, 60):
    x, y = 0.07 * math.cos(math.radians(a)), 0.07 * math.sin(math.radians(a))
    p.box((x, y, 1.82), (0.05, 0.05, 0.14), "straw", top=(0.3, 0.3),
          rot=(math.radians(y * 200), -math.radians(x * 200), 0))
p.box((0, 0, 1.78), (0.10, 0.10, 0.06), "straw_dark")
# braco esquerdo fixo (barra transversal) e ombro direito
p.box((-0.35, 0, 1.40), (0.36, 0.06, 0.06), "wood")
p.box((-0.55, 0, 1.40), (0.08, 0.10, 0.10), "straw")
p.cyl((-0.5, 0, 1.40), 0.04, 0.04, 0.09, "rope", seg=6, rot=(0, math.pi / 2, 0))
p.box((0.12, 0, 1.40), (0.12, 0.08, 0.08), "wood_dark")
dummy = p.to_object("dummy")

# braco direito articulavel: pivo no ombro (x=+0.18), estendido em +X em repouso
a = Part(pal)
a.box((0.20, 0, 0), (0.40, 0.06, 0.06), "wood")
a.box((0.44, 0, 0), (0.08, 0.10, 0.10), "straw")
a.cyl((0.40, 0, 0), 0.04, 0.04, 0.09, "rope", seg=6, rot=(0, math.pi / 2, 0))
a.sphere((0, 0, 0), (0.05, 0.05, 0.05), "wood_dark", u=6, v=4)
arm = a.to_object("arm", (0.18, 0, 1.40), dummy)

finish("dummy", [dummy, arm], out_path(), (0.7, 1.0, 0.3), size=(560, 900))
