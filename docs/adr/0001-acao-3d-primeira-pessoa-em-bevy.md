# Ação 3D em primeira pessoa, em tempo real, em Bevy, com o game feel em primeiro lugar

Apesar de ser um roguelike, o jogo **não** é por turnos nem em grade. É ação em tempo real em primeira pessoa, com visual low-poly no estilo PSX, feito em Bevy (Rust, desktop). A referência de combate é Dark Messiah: peso, impacto e física. O game feel é o critério principal de design: cada mecânica nova precisa ser gostosa de usar antes de a estrutura roguelike crescer em volta dela.

## Opções consideradas

- **Por turnos em grade, em 1ª pessoa** (estilo Grimrock): mais simples e mais "roguelike clássico", mas rejeitado porque o objetivo é a sensação de combate do TES/Dark Messiah.
- **Pixel art top-down 2D**: mais barato de produzir, mas perde o clima de exploração em 1ª pessoa do TES.
- **1ª + 3ª pessoa alternáveis** (estilo Oblivion): dobra o trabalho de animação e de ajuste do game feel num projeto de hobby. Só 1ª pessoa.

## Consequências

- O primeiro marco de desenvolvimento é um protótipo de combate numa sala vazia (uma espada e um inimigo humanoide), e não a estrutura do jogo.
- Física, animação de viewmodel, hitstop, sacudida de câmera e áudio de impacto são responsabilidades centrais desde o início, não acabamento.
