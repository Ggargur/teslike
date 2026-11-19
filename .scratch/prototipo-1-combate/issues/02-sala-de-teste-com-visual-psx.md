# 02: Sala de teste com visual PSX

**What to build:** a sala de teste renderizada com cara de PlayStation 1. Chão, paredes, pilares, uma borda e espetos (usados depois pelo Chute), vistos em baixa resolução, com tremidinha de vértices, texturas sem filtro, névoa e dithering.

**Blocked by:** 01 (Fundação do workspace)

**Status:** ready-for-agent

- [ ] Render numa resolução interna baixa (por exemplo 320×240) ampliada com nearest-neighbor
- [ ] Material com vertex snapping, texturas sem filtro, névoa por distância e dithering ou quantização de cor
- [ ] Todo o visual PSX isolado num plugin próprio (`psx`), em poucos arquivos
- [ ] Parâmetros do PSX (resolução, força do snapping, névoa) ajustáveis pelo `.ron`
- [ ] Sala com chão, paredes, pilares, uma borda e espetos, todos com colisores
- [ ] Um screenshot parece de PS1
