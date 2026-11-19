# 10: Chute e espetos

**What to build:** o Chute empurra o boneco fisicamente, quebra o Bloqueio dele e pode jogá-lo pela borda ou contra os espetos.

**Blocked by:** 07 (Bloqueio, Quebra de Guarda e boneco que ataca)

**Status:** ready-for-agent

- [ ] `rules` (TDD): Chute tem custo de stamina e gera `GuardBroken` contra um Bloqueio
- [ ] Chute aplica impulso físico no alvo
- [ ] Alvo que cai pela borda ou bate nos espetos sofre o efeito (dano ou morte)
- [ ] Animação do Chute e game feel próprios (impacto, câmera)
- [ ] Força do impulso e dano dos espetos no `.ron`
- [ ] Chutar o boneco nos espetos dá vontade de rir (validado jogando)
