import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *

clear_scene()
colors = dict(SWORD_COLORS)
colors.update({
    "skin": "#d9a679", "skin_dark": "#b3835b", "armor": "#3b3f48", "armor_light": "#59606c",
    "cloth": "#2b2622", "crest": "#9c1f1f", "eye": "#f4f0d8", "boot": "#241a14",
})
pal = Palette(colors)
P = lambda: Part(pal)

# ---- pelvis (origem no quadril, 0.95 m) ----
p = P()
p.box((0, 0, -0.03), (0.34, 0.20, 0.17), "cloth", top=(1.05, 1.05))
p.box((0, 0, 0.055), (0.36, 0.22, 0.045), "leather_dark")
p.box((0, 0.112, 0.055), (0.06, 0.012, 0.04), "gold")
p.box((0, 0.105, -0.16), (0.14, 0.02, 0.16), "leather", top=(0.8, 1))   # saiote frontal
pelvis = p.to_object("pelvis", (0, 0, 0.95))

# ---- torso (origem na cintura) ----
p = P()
p.box((0, 0, 0.23), (0.34, 0.21, 0.46), "armor", top=(1.28, 1.15))
p.box((0, 0.118, 0.27), (0.30, 0.03, 0.30), "armor_light", top=(1.15, 1))   # peitoral
p.box((0, 0.118, 0.10), (0.30, 0.03, 0.08), "leather")
p.box((0, 0.0, 0.47), (0.26, 0.17, 0.04), "armor_light")                   # gola
torso = p.to_object("torso", (0, 0, 0.06), pelvis)

# ---- cabeca (origem no pescoco) ----
p = P()
p.cyl((0, 0, 0.03), 0.045, 0.05, 0.08, "skin_dark", seg=6)
p.box((0, 0, 0.15), (0.17, 0.19, 0.18), "skin", top=(1.0, 0.95))
p.box((0, 0.0, 0.225), (0.20, 0.23, 0.10), "metal" if False else "steel_dark", top=(0.9, 0.9))   # elmo
p.box((0, 0.0, 0.29), (0.03, 0.20, 0.05), "crest", top=(1, 0.8))
p.box((0, 0.098, 0.17), (0.03, 0.02, 0.10), "steel_dark")                   # protetor de nariz
for sx in (-1, 1):
    p.box((sx * 0.04, 0.095, 0.185), (0.035, 0.012, 0.02), "eye")
    p.box((sx * 0.105, 0.0, 0.15), (0.02, 0.15, 0.10), "steel_dark")        # bochechas
head = p.to_object("head", (0, 0, 0.48), torso)

# ---- bracos ----
def arm(side, name_up, name_lo, with_sword):
    sx = 1 if side == "r" else -1
    p = P()
    p.box((sx * 0.015, 0, 0.03), (0.15, 0.15, 0.08), "armor_light", top=(0.9, 0.9))    # ombreira
    p.box((0, 0, -0.15), (0.09, 0.09, 0.30), "cloth", top=(1.2, 1.2))
    up = p.to_object(name_up, (sx * 0.25, 0, 0.40), torso)
    p = P()
    p.box((0, 0, -0.10), (0.085, 0.085, 0.20), "leather", top=(1.25, 1.25))
    p.box((0, 0, -0.14), (0.095, 0.095, 0.10), "armor", top=(1, 1))                      # braçadeira
    p.box((0, 0, -0.30), (0.075, 0.08, 0.09), "skin")                                     # mao
    lo = p.to_object(name_lo, (0, 0, -0.30), up)
    sw = None
    if with_sword:
        p = P()
        build_sword(p, scale=0.85, flip=True, lite=True)
        sw = p.to_object("sword", (0, 0, -0.30), lo)
    return up, lo, sw

arm("l", "upper_arm_l", "lower_arm_l", False)
_, _, sword = arm("r", "upper_arm_r", "lower_arm_r", True)

# ---- pernas ----
def leg(side, name_th, name_sh):
    sx = 1 if side == "r" else -1
    p = P()
    p.box((0, 0, -0.22), (0.15, 0.16, 0.44), "cloth", top=(1.2, 1.1))
    th = p.to_object(name_th, (sx * 0.10, 0, -0.02), pelvis)
    p = P()
    p.box((0, 0.03, 0.0), (0.12, 0.05, 0.10), "armor_light")                # joelheira
    p.box((0, 0, -0.22), (0.11, 0.12, 0.44), "boot", top=(1.15, 1.1))
    p.box((0, 0.04, -0.45), (0.13, 0.27, 0.10), "boot")                      # bota
    p.box((0, 0.17, -0.455), (0.12, 0.03, 0.08), "leather_dark")
    p.to_object(name_sh, (0, 0, -0.43), th)
    return th

leg("l", "thigh_l", "shin_l")
leg("r", "thigh_r", "shin_r")

objs = [o for o in bpy.context.scene.objects]
finish("humanoid", objs, out_path(), (0.6, 1.0, 0.25), size=(560, 900))
