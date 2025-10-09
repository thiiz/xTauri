# Requirements Document

## Introduction

Esta feature implementa um sistema de cache local no backend Tauri para armazenar conteúdos Xtream (canais, filmes, séries e categorias) em tabelas SQLite. Atualmente, o frontend faz requisições diretas ao servidor Xtream toda vez que precisa listar conteúdos, o que causa:

1. **Latência alta**: Cada navegação requer uma nova requisição HTTP
2. **Uso excessivo de banda**: Dados são baixados repetidamente
3. **Experiência ruim offline**: Sem conexão, nada funciona
4. **Carga no servidor**: Requisições desnecessárias ao provedor Xtream

Com o cache local, os conteúdos serão:
- Baixados uma vez quando o usuário adiciona/atualiza a conta
- Armazenados em tabelas SQLite otimizadas
- Servidos instantaneamente do banco local
- Atualizados em background periodicamente

## Requirements

### Requirement 1: Database Schema for Content Storage

**User Story:** Como desenvolvedor, eu quero um schema de banco de dados otimizado para armazenar conteúdos Xtream, para que possamos fazer queries rápidas e eficientes.

#### Acceptance Criteria

1. WHEN o sistema inicializa THEN SHALL criar as seguintes tabelas se não existirem:
   - `xtream_channels` - Armazena canais de TV ao vivo
   - `xtream_movies` - Armazena filmes
   - `xtream_series` - Armazena séries
   - `xtream_seasons` - Armazena temporadas de séries
   - `xtream_episodes` - Armazena episódios
   - `xtream_channel_categories` - Categorias de canais
   - `xtream_movie_categories` - Categorias de filmes
   - `xtream_series_categories` - Categorias de séries
   - `xtream_content_sync` - Metadados de sincronização

2. WHEN uma tabela é criada THEN SHALL incluir:
   - Chave primária apropriada
   - Índices para campos frequentemente consultados (profile_id, category_id, name)
   - Foreign keys para relacionamentos
   - Campos de timestamp (created_at, updated_at)
   - Campo profile_id para isolar dados por conta

3. WHEN armazenando dados THEN SHALL usar tipos de dados apropriados:
   - INTEGER para IDs e números
   - TEXT para strings e JSON
   - REAL para ratings
   - TIMESTAMP para datas

### Requirement 2: Content Synchronization on Profile Addition

**User Story:** Como usuário, eu quero que os conteúdos sejam baixados automaticamente quando adiciono uma conta Xtream, para que eu possa navegar rapidamente depois.

#### Acceptance Criteria

1. WHEN o usuário adiciona uma nova conta Xtream THEN SHALL:
   - Iniciar sincronização automática em background
   - Mostrar progresso da sincronização (0-100%)
   - Permitir que o usuário continue usando o app durante a sincronização
   - Notificar quando a sincronização estiver completa

2. WHEN a sincronização está em progresso THEN SHALL baixar na seguinte ordem:
   - Categorias de canais (rápido)
   - Categorias de filmes (rápido)
   - Categorias de séries (rápido)
   - Canais (pode ser lento)
   - Filmes (pode ser lento)
   - Séries (pode ser lento)

3. WHEN ocorre um erro durante a sincronização THEN SHALL:
   - Registrar o erro no log
   - Continuar com os próximos itens
   - Marcar a sincronização como parcial
   - Permitir retry manual

4. WHEN a sincronização é concluída THEN SHALL:
   - Atualizar o timestamp de última sincronização
   - Marcar o perfil como sincronizado
   - Notificar o frontend

### Requirement 3: Local-First Content Retrieval

**User Story:** Como usuário, eu quero que os conteúdos sejam carregados instantaneamente do cache local, para que eu tenha uma experiência rápida e fluida.

#### Acceptance Criteria

1. WHEN o frontend solicita lista de canais THEN SHALL:
   - Buscar primeiro no cache local
   - Retornar dados imediatamente se disponíveis
   - Aplicar filtros e ordenação no backend
   - Não fazer requisição ao servidor Xtream

2. WHEN o frontend solicita lista de filmes THEN SHALL:
   - Buscar no cache local por profile_id
   - Filtrar por categoria se especificado
   - Ordenar por nome, rating ou data
   - Retornar paginado se solicitado

3. WHEN o frontend solicita lista de séries THEN SHALL:
   - Buscar no cache local
   - Incluir contagem de temporadas/episódios
   - Aplicar filtros de categoria
   - Retornar dados estruturados

4. WHEN o frontend solicita detalhes de série THEN SHALL:
   - Buscar série, temporadas e episódios do cache
   - Montar estrutura completa com relacionamentos
   - Retornar em formato otimizado

5. WHEN não há dados no cache THEN SHALL:
   - Retornar lista vazia
   - Indicar que sincronização é necessária
   - Sugerir ao usuário fazer sync manual

### Requirement 4: Background Sync and Updates (Configurable)

**User Story:** Como usuário, eu quero controlar se os conteúdos são atualizados automaticamente em background, para que eu possa escolher entre conveniência e controle de recursos.

#### Acceptance Criteria

1. WHEN o usuário acessa configurações THEN SHALL ver opções de sincronização:
   - Habilitar/desabilitar sync automático em background
   - Intervalo de sincronização (6h, 12h, 24h, 48h, manual)
   - Sincronizar apenas em WiFi (sim/não)
   - Notificar quando sync completo (sim/não)

2. WHEN sync automático está habilitado E última sincronização passou do intervalo configurado THEN SHALL:
   - Verificar se está em WiFi (se configurado)
   - Iniciar sincronização incremental em background
   - Não bloquear a UI
   - Atualizar apenas conteúdos modificados se possível
   - Notificar discretamente quando completo (se configurado)

3. WHEN sync automático está desabilitado THEN SHALL:
   - Não fazer sincronizações automáticas
   - Mostrar indicador de "dados podem estar desatualizados"
   - Permitir apenas sync manual
   - Manter última data de sincronização visível

4. WHEN fazendo sincronização incremental THEN SHALL:
   - Comparar timestamps com servidor
   - Baixar apenas novos conteúdos
   - Atualizar conteúdos modificados
   - Remover conteúdos deletados

5. WHEN o usuário solicita refresh manual THEN SHALL:
   - Mostrar indicador de loading
   - Fazer sincronização completa
   - Atualizar UI quando completo
   - Mostrar mensagem de sucesso
   - Funcionar independente das configurações de sync automático

### Requirement 5: Search and Filter Performance

**User Story:** Como usuário, eu quero que a busca e filtros sejam instantâneos, para que eu possa encontrar conteúdos rapidamente.

#### Acceptance Criteria

1. WHEN o usuário digita na busca THEN SHALL:
   - Fazer busca fuzzy no cache local
   - Retornar resultados em menos de 100ms
   - Buscar em múltiplos campos (nome, descrição, gênero)
   - Ordenar por relevância

2. WHEN o usuário aplica filtros THEN SHALL:
   - Aplicar filtros no backend SQLite
   - Usar índices para performance
   - Combinar múltiplos filtros (categoria, ano, rating)
   - Retornar resultados paginados

3. WHEN fazendo busca em grande volume de dados THEN SHALL:
   - Usar FTS (Full-Text Search) do SQLite
   - Limitar resultados a 1000 itens
   - Implementar paginação eficiente
   - Manter UI responsiva

### Requirement 6: Cache Management and Storage Settings

**User Story:** Como usuário, eu quero controlar o cache local e configurações de sincronização, para que eu possa gerenciar o espaço em disco, uso de banda e atualizar dados quando necessário.

#### Acceptance Criteria

1. WHEN o usuário acessa configurações de cache THEN SHALL mostrar:
   - **Informações do Cache:**
     - Tamanho total do cache
     - Data da última sincronização
     - Número de itens em cache (canais, filmes, séries)
   - **Configurações de Sincronização:**
     - Toggle: Habilitar sync automático em background
     - Dropdown: Intervalo de sincronização (6h, 12h, 24h, 48h, manual)
     - Toggle: Sincronizar apenas em WiFi
     - Toggle: Notificar quando sincronização completa
   - **Ações:**
     - Botão: Sincronizar agora (manual)
     - Botão: Limpar cache
     - Botão: Restaurar configurações padrão

2. WHEN o usuário limpa o cache THEN SHALL:
   - Mostrar diálogo de confirmação com aviso
   - Remover todos os dados de conteúdo
   - Manter dados de perfil e configurações de sync
   - Confirmar ação com o usuário
   - Sugerir nova sincronização
   - Atualizar estatísticas de cache

3. WHEN o cache atinge limite de tamanho (ex: 2GB) THEN SHALL:
   - Alertar o usuário
   - Oferecer opção de limpar dados antigos
   - Manter dados essenciais (categorias e favoritos)
   - Sugerir desabilitar sync automático se necessário

4. WHEN o usuário muda configurações de sync THEN SHALL:
   - Salvar imediatamente no backend
   - Aplicar novas configurações na próxima verificação
   - Mostrar feedback visual de salvamento
   - Validar valores (ex: intervalo mínimo de 6h)

### Requirement 7: Offline Support

**User Story:** Como usuário, eu quero acessar conteúdos mesmo sem internet, para que eu possa navegar minha biblioteca offline.

#### Acceptance Criteria

1. WHEN não há conexão com internet THEN SHALL:
   - Servir todos os dados do cache local
   - Mostrar indicador de modo offline
   - Permitir navegação completa
   - Desabilitar apenas streaming de vídeo

2. WHEN a conexão é restaurada THEN SHALL:
   - Verificar se há atualizações disponíveis
   - Oferecer sincronização automática
   - Remover indicador de offline

### Requirement 8: Migration and Compatibility

**User Story:** Como desenvolvedor, eu quero que a migração seja transparente para usuários existentes, para que não haja quebra de funcionalidade.

#### Acceptance Criteria

1. WHEN um usuário existente atualiza o app THEN SHALL:
   - Detectar perfis sem cache
   - Oferecer sincronização inicial
   - Manter funcionalidade existente até sync completo
   - Migrar gradualmente para cache local

2. WHEN há mudança no schema THEN SHALL:
   - Executar migrations automaticamente
   - Preservar dados existentes
   - Fazer backup antes de migrar
   - Reverter em caso de erro

### Requirement 9: Performance Metrics

**User Story:** Como desenvolvedor, eu quero métricas de performance do cache, para que eu possa otimizar e monitorar o sistema.

#### Acceptance Criteria

1. WHEN fazendo queries no cache THEN SHALL registrar:
   - Tempo de resposta
   - Número de resultados
   - Tipo de query (busca, filtro, listagem)
   - Cache hit/miss ratio

2. WHEN a performance está abaixo do esperado THEN SHALL:
   - Identificar queries lentas
   - Sugerir otimizações (índices, vacuum)
   - Alertar desenvolvedor via logs

### Requirement 10: Data Consistency and Integrity

**User Story:** Como usuário, eu quero que os dados no cache sejam sempre consistentes e confiáveis, para que eu não veja informações incorretas ou desatualizadas.

#### Acceptance Criteria

1. WHEN salvando dados no cache THEN SHALL:
   - Validar estrutura dos dados
   - Usar transações para atomicidade
   - Verificar integridade referencial
   - Fazer rollback em caso de erro

2. WHEN detectando inconsistência THEN SHALL:
   - Registrar erro detalhado
   - Marcar dados como inválidos
   - Sugerir re-sincronização
   - Não corromper dados existentes

3. WHEN múltiplos perfis existem THEN SHALL:
   - Isolar dados por profile_id
   - Prevenir vazamento de dados entre perfis
   - Limpar dados ao deletar perfil
