import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *

clear_scene()
pal = Palette(SWORD_COLORS)
p = Part(pal)
build_sword(p)
sword = p.to_object("sword")
# camera de 3/4 vendo a lamina inteira (lamina sobe)
finish("sword", [sword], out_path(), (0.9, -1.0, 0.35), size=(480, 900))
