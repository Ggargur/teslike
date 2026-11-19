# 04: Ataque leve no boneco (tracer bullet)

**What to build:** o primeiro caminho completo do combate. O jogador segura uma espada, dá um ataque leve, a lâmina acerta um boneco de treino e o boneco reage fisicamente, com hitstop e som. As regras ficam na `rules`, feitas com TDD; o `game` só traduz intenções em entrada e eventos em sensação.

**Blocked by:** 03 (Andar pela sala)

**Status:** ready-for-agent

- [ ] `rules` (TDD): stamina com gasto por ação; máquina de estados da ação preparação → ativo → recuperação; um ataque leve que acerta durante a janela ativa produz `Hit { damage }`
- [ ] Interface da `rules`: recebe intenções + tempo e devolve eventos
- [ ] Espada gerada por script do Blender (300–800 triângulos, textura de paleta 64×64), com prévia PNG
- [ ] Viewmodel procedural: posições-chave e molas para o balanço e o atraso ao virar a câmera, com parâmetros no `.ron`
- [ ] Varredura da lâmina por shape cast do volume varrido entre frames (golpes rápidos não atravessam o alvo)
- [ ] Boneco de treino que reage fisicamente ao acerto
- [ ] Hitstop e um som de impacto (CC0) no acerto
