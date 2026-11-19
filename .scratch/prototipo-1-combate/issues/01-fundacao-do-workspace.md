# 01: Fundação do workspace

**What to build:** o esqueleto do projeto pronto para iterar rápido: um workspace com a crate pura `rules` (sem Bevy) e o app `game`. Abrir o jogo mostra uma janela; editar um arquivo de config `.ron` muda algo visível na tela sem recompilar; o inspector liga e desliga por uma tecla.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Versões de avian3d, leafwing-input-manager e bevy-inspector-egui compatíveis com o Bevy 0.20 foram verificadas antes de fixar (se faltar alguma, registrar a decisão: recuar o Bevy ou trocar a crate)
- [ ] Perfil dev rápido: dependências com `opt-level = 3`, `dynamic_linking` do Bevy, mold se disponível
- [ ] Carregador de config `.ron` com hot reload; editar o arquivo com o jogo aberto muda algo na tela
- [ ] Inspector alterna com uma tecla
- [ ] `cargo test -p rules` roda (com pelo menos um teste trivial) e não depende de Bevy
