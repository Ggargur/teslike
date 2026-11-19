# 07: Bloqueio, Quebra de Guarda e boneco que ataca

**What to build:** o boneco de treino passa a atacar num ritmo fixo, com Telegrafia clara, e o jogador pode segurar o golpe com Bloqueio. Bloquear gasta stamina; sem stamina, ou contra um golpe carregado, a guarda quebra. O jogador passa a ter vida e pode tomar dano.

**Blocked by:** 06 (Ataque carregado)

**Status:** ready-for-agent

- [ ] `rules` (TDD): Bloqueio segura o golpe e gasta stamina; regeneração de stamina com atraso depois de gastar
- [ ] `rules` (TDD): `GuardBroken` quando a stamina zera durante um Bloqueio e quando um golpe carregado acerta um Bloqueio; estado atordoado depois da Quebra de Guarda
- [ ] Vida do jogador; um golpe não bloqueado causa dano
- [ ] Boneco que ataca num ritmo fixo (ajustável no `.ron`), com Telegrafia visual e sonora
- [ ] Feedback de Bloqueio e de Quebra de Guarda (som, câmera, viewmodel)
- [ ] O boneco usa a mesma `rules` que o jogador
