# Protótipo 2: uma Masmorra

**Objetivo**: responder a uma única pergunta: *atravessar uma Masmorra gerada, montando a build com Itens no caminho, é divertido?*

**Critério de pronto**: jogar **10 Masmorras seguidas** e terminar pelo menos 3 delas com builds que pareçam diferentes entre si.

**Fora do escopo**: Mapa de Caminhos, Refúgio, Legado, Habilidades, Eras, Antigos, Vila, Santuário, Magia e save. Ver [ADR 0002](../adr/0002-tentativas-giram-em-torno-de-itens-com-aspectos.md) e o [GLOSSARY](../../GLOSSARY.md).

---

## Decisões do protótipo

| Tema | Decisão |
|---|---|
| Planta | Grafo simples gerado: 3 a 6 Câmaras em sequência, com becos opcionais. Sem WFC nesta escala |
| Câmara | WFC sobre um grid de módulos low-poly (forma + decoração), com restrições: portas conectadas, espaço mínimo de luta e pelo menos um perigo para o Chute |
| Portas | Nunca trancam o jogador |
| Papéis | Duelista (já existe), Bruto e Enxame. Atirador e Conjurador ficam para depois |
| Ataque | Token de ataque: no máximo 2 inimigos atacando ao mesmo tempo |
| Elite | 1 por Masmorra, carrega um Item, usa o efeito dele e o deixa cair ao morrer |
| Itens | 8 slots nomeados de armadura mais 2 mãos. Cada peça tem defesa, peso, 1 efeito gatilho → consequência, raridade e 0 a 2 Aspectos |
| Aspectos | 3 provisórios, com bônus nos limiares de 2 e 4 peças |
| Inventário | Tela em homenagem ao Diablo |
| Cura | Cálice com poucas cargas (sem Santuário nesta etapa, então ele não recarrega) e poções achadas |
| Juice | Decalques de corte, sangue no chão e desmembramento em golpe carregado letal e em morte por espetos |

---

## Marcos

Cada marco termina com algo jogável. Só se avança quando o marco atual está bom de verdade.

### M0: Juice de sangue
- Decalques de corte no inimigo e de sangue no chão, e desmembramento estilizado no visual PSX.
- **Pronto quando**: um golpe carregado letal no humanoide é o melhor momento do protótipo 1.

### M1: Câmara por WFC
- Conjunto de módulos (script no Blender), regras de adjacência e solver com as restrições acima.
- **Pronto quando**: 20 Câmaras geradas seguidas são todas jogáveis, e pelo menos metade delas tem algo interessante para usar o Chute.

### M2: Planta da Masmorra
- Grafo de Câmaras, portas, becos com saque e uma Câmara final com o encontro mais forte.
- **Pronto quando**: dá para atravessar a Masmorra inteira com o humanoide espalhado pelas Câmaras.

### M3: Bruto, Enxame e token de ataque
- Os dois Papéis novos usam a mesma crate `rules`. O Bruto tem golpes que não dá para bloquear, e o Enxame é fraco e anda em grupo.
- **Pronto quando**: uma Câmara com Duelista + Enxame é legível e justa.

### M4: Itens e inventário (crate `rules`, TDD)
- Modelo de Item, slots, peso total afetando esquiva e stamina, gatilhos, Aspectos e limiares.
- Uns 15 Itens de teste e a tela de inventário.
- **Pronto quando**: duas combinações diferentes de Itens mudam de verdade o jeito de lutar.

### M5: Elite e saque
- Elite com Item, tabela de saque por Câmara e o orçamento de Itens ajustado para uma Masmorra.
- **Pronto quando**: ver um Elite usar um efeito dá vontade de pegar o Item dele.

### M6: Loop e ajuste
- Recomeço rápido (Masmorra nova a cada morte ou vitória) e sessões de playtest.
- **Pronto quando**: o critério de pronto do protótipo é cumprido.
