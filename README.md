# Enkai

Uma ferramenta TUI para resolver conflitos git durante merge ou rebase.

## Instalação

```bash
cargo install --path .
```

## Uso

Quando tiver conflitos git:

```bash
enkai
```

## Comandos

### Lista de Arquivos
- `j/k` ou `↑/↓` - Navegar
- `Tab` - Alternar para visualização de código
- `q` - Sair

### Visualização de Código
- `j/k` ou `↑/↓` - Próximo/anterior conflito
- `Ctrl+d/Ctrl+u` - **Scroll** para baixo/cima
- `c` - Aceitar **Current** (HEAD) para o conflito atual
- `i` - Aceitar **Incoming** para o conflito atual
- `b` - Aceitar **Both** para o conflito atual
- `u` - **Desfazer** resolução do conflito atual
- `s` - **Salvar** arquivo (após resolver todos os conflitos)
- `Tab` - Voltar para lista de arquivos
- `q` - Sair

### Após Resolver (Rebase)
- `c` - Continuar rebase
- `a` - Abortar rebase
- `s` - Pular commit

## Como Funciona

1. Detecta conflitos git no repositório
2. Mostra lista de arquivos com conflitos
3. Use `Tab` para ir para a visualização de código
4. Navegue pelos conflitos com `j/k`
5. Para cada conflito, escolha: `c` (Current), `i` (Incoming) ou `b` (Both)
6. Quando todos os conflitos estiverem resolvidos, pressione `s` para salvar
7. Vá para o próximo arquivo ou, se for rebase, escolha continuar/abortar/pular

## Estrutura

```
src/
├── domain/     # Modelos de dados
├── app/        # Estado da aplicação
├── git/        # Integração com git
└── tui/        # Interface do usuário
```

## Desenvolvimento

```bash
# Build
cargo build --release

# Testes
cargo test

# Lint
cargo clippy
```

## Licença

MIT
