# Protótipo 1: combate em uma sala

**Objetivo**: responder uma única pergunta: *o combate corpo a corpo em primeira pessoa é gostoso?*

**Critério de pronto**: lutar **20 vezes seguidas** contra o mesmo inimigo humanoide e continuar querendo lutar a 21ª.

**Fora do escopo**: Magia, Mapa de Caminhos, Refúgio, Legado, Habilidades, inventário, menus, save. Ver [ADR 0001](../adr/0001-acao-3d-primeira-pessoa-em-bevy.md) e o [GLOSSARY](../../GLOSSARY.md).

---

## Decisões do protótipo

| Tema | Decisão |
|---|---|
| Engine | Bevy 0.20 (Rust, desktop) |
| Física | avian3d. O acerto é uma **varredura da lâmina** (shape cast ao longo da espada na janela ativa) |
| Input | leafwing-input-manager, mapeado por ação. Teclado, mouse e **controle** desde o início |
| Visual | Primitivas e modelos low-poly gerados por script no Blender, com shader PSX desde o dia 1 |
| Animação | Procedural: posições-chave, molas e curvas no código. Vale pro viewmodel e pro inimigo |
| Ajuste de números | Arquivos `.ron` com hot reload, mais o bevy-inspector-egui ao vivo |
| Testes | As regras de combate ficam numa crate pura sem Bevy, feita com TDD. O game feel é validado jogando |
| Verbos | Ataque leve, ataque carregado, Bloqueio, Aparo, esquiva, **Chute**, stamina |

## Estrutura do código

```
teslike/
├── Cargo.toml                 # workspace
├── crates/
│   ├── rules/                 # lógica pura de combate, sem Bevy, 100% testável
│   └── game/                  # app Bevy: render, input, física, game feel
├── assets/
│   ├── config/*.ron           # números ajustáveis com hot reload
│   ├── models/*.glb           # gerados pelos scripts do Blender, nunca editados à mão
│   └── sounds/
└── tools/blender/*.py         # um script por modelo; exporta .glb e renderiza a prévia
```

**Por que separar `rules`**: sem Bevy, os testes compilam e rodam em segundos. E a fronteira fica clara: `rules` decide *o que acontece* (o golpe foi aparado, a guarda quebrou), `game` decide *como isso é sentido* (hitstop, sacudida da câmera, faísca, som).

---

## Marcos

Cada marco termina com algo jogável ou visível. Só se avança quando o marco atual está bom de verdade.

### M0: Fundação
- Workspace com `rules` + `game`. Escolher versões de avian3d, leafwing e inspector **compatíveis com o Bevy 0.20** (verificar antes de fixar).
- Perfil dev rápido: dependências com `opt-level = 3`, `dynamic_linking` do Bevy, linker rápido (mold) se estiver disponível.
- Carregador de config `.ron` com hot reload, e o inspector ligando e desligando com uma tecla.
- **Pronto quando**: abre a janela, editar um `.ron` muda algo na tela sem recompilar, e `cargo test -p rules` roda.

### M1: O visual PSX
- Renderizar numa resolução interna baixa (por exemplo 320×240) e ampliar com nearest-neighbor.
- Material com **vertex snapping** (tremidinha PS1), texturas sem filtro, névoa por distância e dithering ou quantização de cor.
- Sala de teste com primitivas: chão, paredes, pilares, uma borda e espetos (pro Chute).
- **Pronto quando**: um screenshot parece de PS1.

### M2: Corpo em primeira pessoa
- Controlador de personagem cinemático no avian3d: andar, correr, mirar com mouse e analógico, colisão com paredes e degraus.
- Balanço de cabeça (head bob), inclinação leve da câmera ao andar de lado, peso na aceleração e na frenagem.
- Mapa de ações no leafwing pra todos os verbos (mesmo os que ainda não existem).
- **Pronto quando**: andar pela sala vazia já é agradável, no teclado e no controle.

### M3: Regras de combate (crate `rules`, TDD)
Testes primeiro, sem gráfico nenhum:
- **Stamina**: gasto por ação, regeneração com atraso depois de gastar e Quebra de Guarda ao zerar durante um Bloqueio.
- **Máquina de estados da ação**: preparação → ativo → recuperação, carregamento, Bloqueio, esquiva (com invulnerabilidade curta), Chute, atordoado.
- **Resolução de golpe**: acerto, Bloqueio, Aparo (janela de tempo) ou Quebra de Guarda (Chute ou golpe carregado contra Bloqueio).
- Interface: a lógica recebe intenções + tempo e devolve eventos (`Parried`, `GuardBroken`, `Hit { damage }`, …).
- **Pronto quando**: os testes descrevem cada interação do combate e passam.

### M4: A espada e o primeiro golpe (o marco mais importante)
- Modelo de espada (script no Blender) + mão com viewmodel procedural: posições-chave e molas pro balanço e pro atraso ao virar a câmera.
- Ataque leve e carregado ligados à crate `rules`, com a varredura da lâmina na janela ativa.
- Boneco de treino que reage fisicamente.
- **A camada de game feel**: hitstop, sacudida de câmera com trauma, pulso de FOV, piscar do alvo, faísca, som de impacto em camadas e recuo do viewmodel.
- **Pronto quando**: bater no boneco parado já é satisfatório.

### M5: Defesa e Chute
- Bloqueio, Aparo (com recompensa forte: som, câmera lenta curta, faísca grande), esquiva e Chute (empurrão físico, borda e espetos).
- Boneco que ataca num ritmo fixo com Telegrafia clara, pra treinar Aparo.
- **Pronto quando**: aparar o boneco 10 vezes seguidas continua prazeroso, e chutá-lo nos espetos dá vontade de rir.

### M6: O inimigo humanoide
- Modelo low-poly em partes rígidas (script no Blender), com animação procedural.
- IA em máquina de estados: aproximar, rodear, golpes telegrafados (2–3 variações), Bloqueio, reação a Aparo (abertura pra contra-ataque), reação a Chute e morte com impulso físico.
- O inimigo usa **a mesma crate `rules`** que o jogador.
- **Pronto quando**: perder pra ele parece justo e ganhar parece merecido.

### M7: Loop de arena e ajuste
- Recomeço instantâneo (morte ou vitória → nova luta em menos de 2s) e um contador de lutas.
- Sessões de playtest com checklist, ajustando pelos `.ron`.
- **Pronto quando**: o critério de pronto do protótipo é cumprido.

---

## Pipeline de modelos (Blender)

- Um script por modelo em `tools/blender/`, rodado com `blender --background --python <script> -- <saída>`.
- Orçamento: 300–800 triângulos por modelo, uma textura de paleta (64×64) e normais simples.
- Cada script também renderiza uma prévia PNG pra revisão antes de entrar no jogo.
- Os modelos são gerados quando um marco precisa deles (espada no M4, humanoide no M6), não antes.

## Riscos

| Risco | Mitigação |
|---|---|
| Crates do ecossistema atrasadas em relação ao Bevy 0.20 | Verificar no M0. Se faltar alguma, recuar uma versão do Bevy ou trocar a crate |
| A API de shaders e materiais do Bevy muda entre versões | Isolar o PSX num plugin próprio (`psx`), em poucos arquivos |
| Varredura da lâmina errar golpes rápidos (tunneling) | Fazer o cast do *volume varrido* entre frames, não da posição instantânea |
| Animação procedural ficar "robótica" | Molas com overshoot, antecipação e follow-through como parâmetros no `.ron` |
| Som é metade do game feel | Começar com sons CC0 (por exemplo, packs da Kenney) desde o M4 |
