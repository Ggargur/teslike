# 11: Inimigo humanoide que ataca

**What to build:** um inimigo humanoide low-poly substitui o boneco. Ele se aproxima, rodeia o jogador e ataca com 2–3 golpes telegrafados, e pode matá-lo.

**Blocked by:** 07 (Bloqueio, Quebra de Guarda e boneco que ataca)

**Status:** ready-for-agent

- [ ] Modelo em partes rígidas gerado por script do Blender (300–800 triângulos, textura de paleta), com prévia PNG
- [ ] Animação procedural com molas, antecipação e follow-through ajustáveis no `.ron`
- [ ] IA em máquina de estados: aproximar, rodear, atacar
- [ ] 2–3 variações de golpe, cada uma com Telegrafia distinguível
- [ ] O inimigo usa a mesma `rules` que o jogador
- [ ] O jogador pode morrer
