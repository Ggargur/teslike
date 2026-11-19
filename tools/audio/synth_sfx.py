#!/usr/bin/env python3
"""Synthesizes all game SFX (CC0, original) into assets/sounds/*.wav (mono 44.1k 16-bit)."""
import os, wave
import numpy as np

SR = 44100
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "sounds")
rng = np.random.default_rng(1337)

def t(d): return np.arange(int(SR * d)) / SR
def noise(d): return rng.standard_normal(int(SR * d))
def env(d, a=0.002, decay=0.1):
    x = t(d); e = np.exp(-x / decay)
    return e * np.minimum(1, x / max(a, 1e-4))

def lp(x, fc):
    """One-pole lowpass; fc scalar or per-sample array."""
    fc = np.broadcast_to(np.asarray(fc, float), x.shape)
    a = 1 - np.exp(-2 * np.pi * fc / SR)
    y = np.empty_like(x); s = 0.0
    for i in range(len(x)):
        s += a[i] * (x[i] - s); y[i] = s
    return y
def hp(x, fc): return x - lp(x, fc)
def bp(x, fc, w=1.0): return lp(hp(x, fc / (1 + w)), fc * (1 + w))
def sweep(d, f0, f1, k=3.0):
    x = t(d) / d; return f1 + (f0 - f1) * np.exp(-k * x)
def sine_drop(d, f0, f1, decay, k=5.0):
    ph = 2 * np.pi * np.cumsum(sweep(d, f0, f1, k)) / SR
    return np.sin(ph) * env(d, 0.001, decay)
def sat(x, g=2.0): return np.tanh(x * g) / np.tanh(g)
def bell(d, f, decay, ratios=(1, 2.76, 5.4, 8.93), amps=(1, .6, .4, .25)):
    x = t(d); y = 0
    for r, a in zip(ratios, amps):
        y = y + a * np.sin(2 * np.pi * f * r * x + rng.uniform(0, 6)) * np.exp(-x / (decay / (1 + .5 * r)))
    return y
def pad(x, d):
    n = int(SR * d); out = np.zeros(n); out[:min(n, len(x))] = x[:n]; return out
def mix(d, *layers):
    out = np.zeros(int(SR * d))
    for l, g, off in layers:
        o = int(SR * off); seg = l[:len(out) - o]; out[o:o + len(seg)] += g * seg
    return out
def fade(x, ms=5):
    n = int(SR * ms / 1000); x = x.copy(); x[-n:] *= np.linspace(1, 0, n); return x

def whoosh(d, f0, f1, peak, g=1.0):
    x = t(d); u = x / d
    e = np.sin(np.pi * u ** peak) ** 2 if False else (np.sin(np.pi * np.clip(u ** peak, 0, 1)) ** 2)
    n = noise(d)
    return sat(bp(n, sweep(d, f0, f1, 1.5) , 1.2) * e * 6, g)

S = {}
S["swing_light"] = whoosh(.25, 3500, 900, .7) + .3 * whoosh(.25, 1200, 500, .7)
S["swing_heavy"] = whoosh(.45, 1400, 200, .65, 1.5) + .5 * sine_drop(.45, 140, 50, .2) * np.sin(np.pi * t(.45) / .45)
S["hit_thud"] = sat(sine_drop(.28, 130, 38, .09) * 1.2 + .4 * lp(noise(.28), 250) * env(.28, .001, .03) * 4, 1.8)
S["hit_crunch"] = sat(bp(noise(.22), 1800, 1.5) * env(.22, .001, .05) * 5 + bp(noise(.22), 700, 1) * env(.22, .001, .09) * 4
                      + 0.3 * sine_drop(.22, 200, 80, .05), 2)
S["hit_wood"] = sat(sum(a * np.sin(2 * np.pi * f * t(.3)) * np.exp(-t(.3) / dd) for f, a, dd in
                    [(180, 1, .07), (330, .6, .05), (520, .35, .035)]) + bp(noise(.3), 900, 1) * env(.3, .0005, .02) * 3, 1.5)
S["block"] = sat(bell(.45, 620, .18, (1, 1.5, 2.3, 3.7), (1, .5, .5, .3)) * .5 + bp(noise(.45), 2500, 1) * env(.45, .0005, .02) * 3
                 + sine_drop(.45, 150, 70, .06) * .6, 1.5)
S["parry"] = sat(bell(1.2, 1480, .9) * .55 + bell(1.2, 2210, .5, (1, 2.1, 3.3), (.5, .3, .2)) * .4
                 + hp(noise(1.2), 4000) * env(1.2, .0005, .015) * 2, 1.3)
S["guard_break"] = sat(bell(.7, 900, .25, (1, 1.8, 3.1, 4.7), (1, .7, .5, .4)) * .5 + bp(noise(.7), 3000, 1.5) * env(.7, .0005, .04) * 4
                       + sine_drop(.7, 110, 32, .25) * 1.4, 1.8)
S["kick"] = sat(sine_drop(.3, 110, 35, .1) * 1.3 + lp(noise(.3), 400) * env(.3, .001, .02) * 4, 2)
S["spikes"] = sat(bp(noise(.5), 2200, 1.5) * (env(.5, .02, .12)) * 4 * (1 + .5 * np.sin(2 * np.pi * 40 * t(.5)))
                  + bell(.5, 1100, .2, (1, 2.4, 4.1), (.6, .4, .3)) * .5 * np.exp(-t(.5) / .15) * (t(.5) > .05)
                  + sine_drop(.5, 120, 50, .12) * .8, 1.6)
S["death"] = sat(sine_drop(.9, 90, 30, .3, 4) * 1.3 + lp(noise(.9), 300) * env(.9, .001, .12) * 4
                 + mix(.9, (lp(noise(.3), 600) * env(.3, .001, .06) * 3, .8, .25), (sine_drop(.3, 70, 30, .1), .8, .25))[:int(SR * .9)], 1.8)
S["telegraph"] = sat((bp(noise(.4), 500, 1.5) * np.sin(2 * np.pi * 14 * t(.4)) * .5 + .8 * np.sin(2 * np.pi * np.cumsum(sweep(.4, 180, 330, -1.5)) / SR)
                      * (1 + .6 * np.sin(2 * np.pi * 9 * t(.4)))) * np.sin(np.pi * np.clip(t(.4) / .4, 0, 1) ** .6) ** .5 * 2, 2)
u = t(.8) / .8
S["charge"] = sat((np.sin(2 * np.pi * np.cumsum(80 + 420 * u ** 2) / SR) + .5 * np.sin(2 * np.pi * np.cumsum(160 + 840 * u ** 2) / SR)
                   + .3 * lp(noise(.8), 200 + 3000 * u ** 2)) * (.15 + .85 * u ** 1.5) * 1.5, 1.8)
S["dodge"] = whoosh(.3, 2000, 600, .6, .8) * .8 + .3 * lp(noise(.3), 400) * np.sin(np.pi * t(.3) / .3) ** 2
S["hurt"] = sat(sine_drop(.4, 100, 40, .14) * 1.3 + lp(noise(.4), 500) * env(.4, .001, .05) * 4 + bp(noise(.4), 1500, 1) * env(.4, .001, .03) * 2, 2.2)
S["footstep"] = sat(sine_drop(.16, 160, 60, .035) * .9 + bp(noise(.16), 1200, 1.2) * env(.16, .0005, .02) * 2.5 + lp(noise(.16), 300) * env(.16, .001, .03) * 2, 1.5)

os.makedirs(OUT, exist_ok=True)
for name, x in S.items():
    x = fade(np.asarray(x, float)); x -= x.mean()
    x = x / np.max(np.abs(x)) * 0.89
    with wave.open(os.path.join(OUT, name + ".wav"), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes((x * 32767).astype("<i2").tobytes())
    print(f"{name}.wav {len(x)/SR:.2f}s")
