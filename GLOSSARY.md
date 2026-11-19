# Glossário

Linguagem do jogo: um roguelike de ação em primeira pessoa, sobre a Disputa entre o Deus da Persistência e o Deus do Eterno.

O design se discute em português e o código é escrito inteiro em inglês. Cada termo traz o nome canônico que ele recebe no código (`_Código_`).

## Personagens

**Deus da Persistência**
O protagonista controlado pelo jogador. Não é imortal: morre, mas sempre tenta de novo. Esse retorno é a natureza dele.
_Código_: `Persistence`
_Evitar_: herói, jogador (quando se fala do personagem)

**Deus do Eterno**
O chefe final. Ex-amigo do Deus da Persistência. Ainda lúcido no Prólogo, foi enlouquecendo ao longo das Eras.
_Código_: `Eternal`
_Evitar_: vilão, boss (como nome canônico)

**Antigos**
Os seres que atravessam as Eras (vampiros, ninfas, elfos, deuses menores). Só com eles o Deus da Persistência tem Relacionamentos, porque só eles sobrevivem para lembrar.
_Código_: `Ancient`
_Evitar_: NPC (como termo de domínio)

**A Disputa**
A aposta entre os dois deuses: a eternidade sobrevive por mais tempo que a persistência? É o conflito que move todo o jogo.
_Código_: `Wager`

## Estrutura

**Mapa de Caminhos**
O mundo semiaberto: uma rede de Nós ligados por caminhos, em que o jogador escolhe a rota rumo ao Deus do Eterno. Não é exploração livre.
_Código_: `PathMap`
_Evitar_: mundo aberto, overworld

**Região**
Um trecho do Mapa de Caminhos: um conjunto de Nós que termina num Guardião. Várias Regiões levam até o Deus do Eterno. As Regiões e a ordem delas são sempre as mesmas, mas os Nós dentro de cada uma são sorteados a cada Tentativa.
_Código_: `Region`

**Nó**
Uma parada no Mapa de Caminhos. Há cinco tipos: **Masmorra** (combate e saque), **Vila** (loja, personagens, quests), **Santuário** (bênção ou escolha), **Evento** (encontro narrativo) e **Guardião** (chefe que fecha a Região).
_Código_: `Node`. Tipos: `Dungeon`, `Village`, `Shrine`, `Encounter`, `Warden`
_Evitar_: fase, sala, level

**Câmara**
Um espaço dentro de uma Masmorra, ligado a outros por portas. Uma Masmorra é uma sequência curta de Câmaras com becos opcionais, e a última guarda o encontro mais forte e o saque principal. As portas nunca trancam o jogador.
_Código_: `Chamber`
_Evitar_: sala (como termo de domínio), room

**Refúgio**
O hub pequeno onde o Deus da Persistência renasce entre Tentativas.
_Código_: `Refuge`
_Evitar_: hub, base, lobby

**Agravamento**
Um degrau permanente de dificuldade, disparado na primeira vez que um Guardião é derrotado: o Deus do Eterno nota o retorno do Deus da Persistência, e o mundo fica mais hostil dali em diante.
_Código_: `Escalation`

**Tentativa**
Uma jornada do Deus da Persistência rumo ao Deus do Eterno, que termina em morte ou no confronto final. O jogo é uma sequência de Tentativas.
_Código_: `Attempt`
_Evitar_: run, ciclo, vida

**Prólogo**
A primeira Tentativa, roteirizada: o Deus da Persistência enfrenta o Deus do Eterno ainda lúcido, obtém a primeira Magia e morre inevitavelmente. Depois acorda no Refúgio, uma Era mais tarde.
_Código_: `Prologue`
_Evitar_: tutorial

**Era**
O tempo que passa entre uma morte e o retorno seguinte. Cada morte custa uma Era: as Vilas mudam de estado, os mortais dão lugar a outros, e só os Antigos permanecem e se lembram. Não há limite de Eras.
_Código_: `Age`

**Loucura**
O quanto o Deus do Eterno perdeu a cabeça. Sobe a cada Era e muda *como* ele reage ao Deus da Persistência, mas não decide se ainda é possível salvá-lo. Quanto maior a Loucura, menos ele conversa e mais interfere nas Tentativas, e a luta final ganha fases novas.
_Código_: `Madness`

**Desfecho**
Como a Disputa termina: **Salvar** ou **Matar** o Deus do Eterno. Matar está sempre disponível. Salvar é liberado ao reunir as Recordações (provisório, a rever).
_Código_: `Ending`. Variantes: `Save`, `Slay`

**Quest**
Uma cadeia curta de etapas, ligada a um personagem. Cada etapa cabe numa Tentativa, e algumas só podem ser feitas depois que uma Era passa: morrer também serve para avançar.
_Código_: `Quest`

**Recordação**
Um fragmento único da amizade entre os dois deuses, obtido por uma linha de quests. Reunir as Recordações libera o Desfecho Salvar.
_Código_: `Recollection`
_Evitar_: memória (esse nome é da moeda)

## Combate

**Aparo**
Um bloqueio feito no instante exato em que o golpe chega. Não gasta stamina, desequilibra o atacante e abre uma janela para contra-atacar.
_Código_: `Parry`
_Contraste_: **Bloqueio** (`Block`), que segura o golpe a qualquer momento, mas gasta stamina.

**Quebra de Guarda**
Quando um combatente perde a capacidade de bloquear por um instante, por ficar sem stamina, levar um Chute ou receber um golpe carregado.
_Código_: `GuardBreak`

**Chute**
Um empurrão físico que quebra a guarda e pode jogar o alvo contra o ambiente (espetos, bordas).
_Código_: `Kick`

**Telegrafia**
O sinal visual e sonoro de que um golpe vem aí, que permite ao defensor ler o golpe e reagir a tempo.
_Código_: `Telegraph`

**Papel**
O que um inimigo testa no jogador, independente da aparência. Papéis: **Duelista** (Aparo e leitura), **Bruto** (golpes imbloqueáveis, esquiva), **Enxame** (grupo fraco, espaço e Chute), **Atirador** (distância) e **Conjurador** (suporte, prioridade de alvo). Cada Região veste os Papéis com a sua própria aparência.
_Código_: `Role`. Variantes: `Duelist`, `Brute`, `Swarm`, `Archer`, `Caster`
_Evitar_: classe, tipo de inimigo

**Elite**
Um inimigo que carrega um Item de verdade, usa o efeito dele em combate e o deixa cair ao morrer.
_Código_: `Elite`

**Guardião**
O chefe feito à mão que fecha uma Região. Alguns Guardiões são Antigos e podem ser **Poupados** em vez de mortos, e isso pesa nos Relacionamentos e no Desfecho. Cada um tem a sua própria condição para ser Poupado, de combate e narrativa, que o jogador descobre ao longo das Eras.
_Código_: `Warden`. Poupar: `Spare`

## Progressão

**Progressão por Uso**
Uma habilidade melhora quando é usada contra uma ameaça real, e quanto maior o perigo, mais ela melhora. É a forma de meta-progressão: o que se ganha assim sobrevive à morte.
_Código_: `UseProgression`
_Evitar_: XP, level up

**Habilidade**
*Como* o Deus da Persistência faz algo: competência, como menos stamina, uma janela de Aparo maior ou uma conjuração mais rápida. Persiste inteira entre Tentativas.
_Código_: `Skill`
_Contraste_: **Item** (`Item`), *o que* ele faz a mais, como poder bruto e efeitos novos. É Espólio.

**Item**
Uma arma, uma armadura ou um consumível. É em torno dos Itens que cada Tentativa ganha identidade: eles mudam regras e combinam entre si. São Espólio.
_Código_: `Item`
_Contraste_: **Magia** (`Spell`) não é Item.

Habilidades (nomes provisórios, em jogo / código): **Gume** / `Edge` (lâminas), **Impacto** / `Impact` (armas contundentes), **Guarda** / `Guard` (bloqueio e Aparo), **Fôlego** / `Breath` (esquiva e stamina), **Ruína** / `Ruin` (magia destrutiva), **Molde** / `Mold` (magia de alteração), **Remendo** / `Mend` (magia de cura).

**Aspecto**
Uma força abstrata do mundo que um personagem encarna: a Persistência (o Deus da Persistência), a Eternidade (o Deus do Eterno) e uma por Antigo importante. Itens trazem Aspectos, e reunir vários do mesmo Aspecto dá bônus. Não é uma classe escolhida: o Aspecto dominante de uma Tentativa surge dos Itens encontrados. Os Itens de Eternidade são fortes, mas cobram um preço que afasta o Desfecho Salvar.
_Código_: `Aspect`
_Evitar_: classe, tag, set

**Bênção**
Um efeito menor que um Item, escolhido num Santuário, que dura só até o fim da Tentativa.
_Código_: `Blessing`
_Contraste_: **Item** (`Item`), que é maior e pode ser trocado.

**Cálice**
O frasco de cura com poucas cargas, que se recarrega nos Santuários. Convive com poções (consumíveis) e com Magias de cura.
_Código_: `Chalice`
_Evitar_: frasco, estus

**Marco**
Um nível de habilidade que libera uma escolha na árvore daquela habilidade: a cada 10 níveis, de 0 a 50. Em cada Marco se escolhe uma entre poucas opções, e a escolha pode ser trocada no Refúgio gastando Memória. Os nós só são liberados por Marcos, nunca por pontos.
_Código_: `Milestone`

**Legado**
O que sobrevive a uma Tentativa: níveis de habilidade, progresso de quests, Relacionamentos e Memória.
_Código_: `Legacy`
_Contraste_: **Espólio** (`Spoils`: armas, armaduras, ouro, Magias e itens de quest) se perde a cada morte. Um item de quest perdido volta pra origem, mas as etapas da quest já cumpridas continuam cumpridas.

**Memória**
A moeda que resiste às Eras. É coletada durante a Tentativa e nunca se perde com a morte. É gasta no Refúgio para adicionar Itens e Magias ao mundo, liberar Kits, trocar escolhas de Marco e melhorar o Refúgio, nunca para comprar poder direto. É Legado.
_Código_: `Memory`
_Contraste_: ouro (`Gold`) é Espólio e só é gasto nas Vilas. O nome do ouro muda de Era para Era, mas o termo de domínio é sempre ouro.

**Kit**
O equipamento com que o Deus da Persistência começa uma Tentativa: uma arma e poucos Itens comuns, às vezes uma Magia, cada Kit puxando para um Aspecto. Escolhe-se um entre os Kits já liberados com Memória.
_Código_: `Kit`
_Evitar_: loadout, classe

**Magia**
Um efeito pronto, específico e marcante, lançado de uma mão. O jogador não cria Magias: ele as encontra durante a Tentativa. Cada Magia tem poucas cargas, que recarregam quando o Deus da Persistência acerta golpes. São Espólio.
_Código_: `Spell`

**Relacionamento**
O vínculo entre o Deus da Persistência e um Antigo. Persiste entre Tentativas, e os Antigos reagem a cada morte e retorno. Avança por estágios nomeados, sem número visível: desconhecido, conhecido, aliado, confidente. Um vínculo forte faz os Itens do Aspecto daquele Antigo aparecerem mais.
_Código_: `Bond`
_Evitar_: relationship, affinity
