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

# ---------------------------------------------------------------- mastering
# Every asset leaves here at the same perceived loudness with true-peak headroom;
# the mix hierarchy (swings under hits, quiet footsteps...) lives in feel.ron.
REF_LUFS = -12.0      # max momentary loudness (100 ms, BS.1770 K-weighted)
CEIL_DBTP = -1.0      # true-peak ceiling (4x oversampled), survives 44.1k->48k resampling
HPF_HZ, LPF_HZ = 35.0, 16000.0
FADE_IN_MS, FADE_OUT_MS, TAIL_DB = 1.0, 12.0, -60.0

def biquad(x, b, a):
    y = np.empty_like(x); x1 = x2 = y1 = y2 = 0.0
    b0, b1, b2 = b; _, a1, a2 = a
    for i in range(len(x)):
        v = x[i]; o = b0 * v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2
        x2, x1, y2, y1 = x1, v, y1, o; y[i] = o
    return y
def butter2(x, fc, high):
    k = np.tan(np.pi * fc / SR); q = 2 ** -.5; n = 1 / (1 + k / q + k * k)
    b = [n, -2 * n, n] if high else [k * k * n, 2 * k * k * n, k * k * n]
    return biquad(x, b, [1, 2 * (k * k - 1) * n, (1 - k / q + k * k) * n])
def k_weight(x):
    k = np.tan(np.pi * 1681.974450955533 / SR); q = .7071752369554196
    vh = 10 ** (3.999843853973347 / 20); vb = vh ** .4996667741545416; a0 = 1 + k / q + k * k
    x = biquad(x, [(vh + vb * k / q + k * k) / a0, 2 * (k * k - vh) / a0, (vh - vb * k / q + k * k) / a0],
               [1, 2 * (k * k - 1) / a0, (1 - k / q + k * k) / a0])
    k = np.tan(np.pi * 38.13547087602444 / SR); q = .5003270373238773; a0 = 1 + k / q + k * k
    return biquad(x, [1, -2, 1], [1, 2 * (k * k - 1) / a0, (1 - k / q + k * k) / a0])
def loudness(x, win=.1):
    p = k_weight(x) ** 2; n = int(SR * win); p = np.pad(p, (0, max(0, n - len(p))))
    return -.691 + 10 * np.log10(np.convolve(p, np.ones(n) / n, "valid").max() + 1e-12)
def upsample4(x):
    pad = 1024; n = len(x) + 2 * pad; X = np.fft.rfft(np.pad(x, pad))
    Y = np.zeros(2 * n + 1, complex); Y[:len(X)] = X
    return np.fft.irfft(Y, 4 * n)[4 * pad:4 * (pad + len(x))] * 4
def true_peak(x): return 20 * np.log10(np.abs(upsample4(x)).max() + 1e-12)
def limit(x, ceil_db, look_ms=1.5, rel_ms=60.0):
    """Lookahead brickwall on the 4x-oversampled envelope; gain glides back over rel_ms."""
    c = 10 ** (ceil_db / 20)
    pk = np.abs(upsample4(x)).reshape(-1, 4).max(1)
    need = np.minimum(1, c / np.maximum(pk, 1e-9))
    la = int(SR * look_ms / 1000)
    need = np.array([need[i:i + la + 1].min() for i in range(len(need))])  # look ahead
    r = np.exp(-1 / (SR * rel_ms / 1000)); g = np.empty_like(need); s = 1.0
    for i in range(len(need)):
        s = need[i] if need[i] < s else need[i] + (s - need[i]) * r; g[i] = s
    return x * g
def master(x):
    x = np.asarray(x, float); x = x - x.mean()
    x = butter2(butter2(x, HPF_HZ, True), LPF_HZ, False)
    tail = np.nonzero(np.abs(x) > 10 ** (TAIL_DB / 20) * np.abs(x).max())[0][-1] + 1
    x = x[:tail]
    n = int(SR * FADE_IN_MS / 1000); x[:n] *= .5 - .5 * np.cos(np.linspace(0, np.pi, n))
    n = min(int(SR * FADE_OUT_MS / 1000), len(x) // 4); x[-n:] *= np.cos(np.linspace(0, np.pi / 2, n)) ** 2
    x = x * 10 ** ((REF_LUFS - loudness(x)) / 20)
    for _ in range(4):  # limiter, then trim any residual over left by the 16-bit grid
        x = limit(x, CEIL_DBTP - .1)
        over = true_peak(x) - CEIL_DBTP
        if over <= 0: break
        x *= 10 ** (-(over + .05) / 20)
    return x

os.makedirs(OUT, exist_ok=True)
print(f"{'sound':12} {'dur':>5} {'LUFS-M':>7} {'dBTP':>6} {'limGR':>5}")
for name, x in S.items():
    y = master(x)
    with wave.open(os.path.join(OUT, name + ".wav"), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes(np.round(np.clip(y, -1, 1) * 32767).astype("<i2").tobytes())
    m = loudness(y)
    print(f"{name:12} {len(y)/SR:5.2f} {m:7.1f} {true_peak(y):6.1f} {REF_LUFS - m:5.1f}")
