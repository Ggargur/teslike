# 09: Esquiva

**What to build:** o jogador pode esquivar do golpe do boneco: um deslocamento curto com invulnerabilidade breve, pago com stamina, com movimento de câmera que vende a sensação.

**Blocked by:** 07 (Bloqueio, Quebra de Guarda e boneco que ataca)

**Status:** ready-for-agent

- [ ] `rules` (TDD): esquiva com custo de stamina e janela de invulnerabilidade; golpe durante a invulnerabilidade não acerta; sem stamina não esquiva
- [ ] Deslocamento físico na direção do input, respeitando colisão
- [ ] Câmera e viewmodel reagem à esquiva
- [ ] Distância, duração e janela de invulnerabilidade no `.ron`
- [ ] Dá pra desviar do ataque telegrafado do boneco de forma consistente
