# 08: Aparo

**What to build:** bloquear no instante exato em que o golpe do boneco chega vira um Aparo: não gasta stamina, desequilibra o atacante e dá uma recompensa forte (câmera lenta curta, faísca grande, som marcante).

**Blocked by:** 07 (Bloqueio, Quebra de Guarda e boneco que ataca)

**Status:** ready-for-agent

- [ ] `rules` (TDD): janela de Aparo no início do Bloqueio; dentro dela o golpe produz `Parried`, sem gasto de stamina; fora dela é Bloqueio normal
- [ ] O atacante aparado fica desequilibrado (estado que abre janela para contra-ataque)
- [ ] Recompensa: câmera lenta curta, faísca grande, som próprio
- [ ] Tamanho da janela e intensidade da recompensa no `.ron`
- [ ] Aparar o boneco 10 vezes seguidas continua prazeroso (validado jogando)
