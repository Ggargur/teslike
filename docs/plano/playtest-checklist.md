# Checklist de playtest: protótipo 1

Para a issue 14. Rode `cargo run -p game`, jogue, anote o que incomodou e ajuste os números em `assets/config/*.ron` (recarregam sozinhos com o jogo aberto).

## Controles

| Verbo | Teclado e mouse | Controle |
|---|---|---|
| Andar / correr | WASD / Shift | Analógico esquerdo / L3 |
| Olhar | Mouse | Analógico direito |
| Ataque leve / carregado | Clique esquerdo (toque / segure) | RT |
| Bloqueio (no instante = Aparo) | Clique direito | LT |
| Esquiva | Espaço | A |
| Chute | F | RB |

F2 boneco · F3 humanoide · R reinicia a luta · F1 inspector · H esconde a ajuda.

## O que observar em cada verbo

| Verbo | Pergunta | Onde ajustar |
|---|---|---|
| Andar | A câmera balança sem enjoar? Frear tem peso? | `player.ron` (`bob_*`, `accel`, `decel`) |
| Ataque leve | O golpe sai na hora? O acerto "trava" o mundo o suficiente? | `combat.ron` (`light`), `feel.ron` (`light_hit`) |
| Ataque carregado | Dá pra sentir a carga crescer? O impacto é claramente mais pesado? | `combat.ron` (`charged`), `viewmodel.ron` (`charge`) |
| Bloqueio | Segurar golpes gasta stamina num ritmo justo? | `combat.ron` (`block.stamina_per_damage`) |
| Aparo | A janela é justa, nem fácil nem impossível? A recompensa é marcante? | `combat.ron` (`block.parry_window`), `feel.ron` (`parried`, `parry_slowmo_*`) |
| Esquiva | Dá pra desviar do golpe telegrafado de forma consistente? | `combat.ron` (`dodge`), `player.ron` (`dodge_speed`) |
| Chute | Chutar nos espetos ou pela borda dá vontade de rir? | `physics.ron` (`kick_force`, `spike_*`) |

## O que observar na luta (humanoide)

- [ ] Cada golpe dele (corte, golpe por cima, estocada) é reconhecível pela Telegrafia antes de chegar.
- [ ] Perder parece justo: dá pra apontar o erro que cometi.
- [ ] Ganhar parece merecido: usei Aparo, Chute ou esquiva, não só cliques.
- [ ] O recomeço é instantâneo e não cansa.
- [ ] Na 20ª luta seguida, ainda quero a 21ª.

Ajustes da IA ficam em `enemy.ron` (`attacks`, `block_chance`, `reaction_time`, `kick_chance`).

## Registro de sessões

| Data | Lutas seguidas | O que incomodou | Ajuste feito |
|---|---|---|---|
| | | | |
