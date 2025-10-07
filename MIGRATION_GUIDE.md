# 🔄 Guia de Migração - Playlists → Xtream Codes

## Para Usuários

### ⚠️ Mudanças Importantes

**Antes (v0.1.x)**
- ✅ Suporte a playlists M3U/M3U8
- ✅ Adicionar playlists por URL ou arquivo
- ✅ Múltiplas playlists
- ✅ Saved filters por playlist

**Depois (v0.2.x)**
- ❌ Playlists M3U não são mais suportadas
- ✅ Apenas contas Xtream Codes
- ✅ Múltiplos perfis Xtream
- ✅ Live TV, Movies, Series, EPG

### 📋 Checklist de Migração

#### Antes de Atualizar
- [ ] Anote suas playlists favoritas
- [ ] Verifique se seu provedor oferece Xtream Codes
- [ ] Obtenha credenciais Xtream (URL, username, password)
- [ ] Faça backup de favoritos (se possível)

#### Após Atualizar
- [ ] Abra o app (mostrará tela de boas-vindas)
- [ ] Clique em "Add Profile"
- [ ] Insira credenciais Xtream
- [ ] Aguarde validação
- [ ] Navegue pelo conteúdo

### 🔑 Obtendo Credenciais Xtream

#### Do Seu Provedor IPTV
1. Entre em contato com seu provedor
2. Solicite credenciais Xtream Codes
3. Você receberá:
   - **URL**: `http://server.com:port`
   - **Username**: seu usuário
   - **Password**: sua senha

#### Exemplo de Credenciais
```
URL: http://example.com:8080
Username: user123
Password: pass456
```

### 🚫 O Que Não Funciona Mais

#### Playlists M3U
```
❌ http://provider.com/playlist.m3u
❌ file:///path/to/playlist.m3u8
❌ Adicionar playlist por URL
❌ Adicionar playlist por arquivo
```

#### Saved Filters
```
❌ Filtros salvos por playlist
❌ Slots de filtros (F1-F5)
```

### ✅ O Que Funciona

#### Xtream Codes
```
✅ Múltiplos perfis Xtream
✅ Live TV (canais ao vivo)
✅ Movies (filmes on-demand)
✅ Series (séries com episódios)
✅ EPG (guia de programação)
✅ Categorias nativas
```

#### Funcionalidades Mantidas
```
✅ Favoritos
✅ Histórico
✅ Busca fuzzy
✅ Navegação por teclado
✅ Player externo (MPV)
✅ Preview de vídeo
```

### 📱 Nova Interface

#### Antes
```
Sidebar:
- Channels
- Favorites
- Groups
- History
- Settings
- Help
```

#### Depois
```
Sidebar:
- Channels      ← Live TV
- Movies        ← Novo!
- Series        ← Novo!
- Favorites
- History
- Groups
- Profiles      ← Novo!
- Help
- Settings
```

### 🎯 Fluxo de Uso

#### Primeira Vez
```
1. Abrir app
2. Ver tela "Welcome to Tollo"
3. Clicar "Add Profile"
4. Inserir credenciais Xtream
5. Aguardar validação
6. Conteúdo carregado automaticamente
```

#### Uso Normal
```
1. Selecionar perfil (topo da sidebar)
2. Navegar entre Channels/Movies/Series
3. Buscar conteúdo
4. Reproduzir
5. Adicionar a favoritos
```

#### Múltiplos Perfis
```
1. Ir em "Profiles"
2. Clicar "Add Profile"
3. Adicionar novo perfil
4. Trocar entre perfis no ProfileSelector
```

## Para Desenvolvedores

### 🔧 Mudanças no Código

#### Antes (Playlists)
```typescript
// Carregar playlist
const channelLists = await invoke('get_channel_lists');
const channels = await invoke('get_channels', { id: listId });

// Adicionar playlist
await invoke('add_channel_list', { name, source });

// Refresh playlist
await invoke('refresh_channel_list', { id });
```

#### Depois (Xtream)
```typescript
// Carregar perfil
const profiles = await invoke('get_xtream_profiles');
const channels = await invoke('get_xtream_channels', { profileId });

// Adicionar perfil
await invoke('create_xtream_profile', { 
  name, url, username, password 
});

// Conteúdo sempre atualizado (API em tempo real)
```

### 📦 Stores Modificados

#### channelStore.ts
```typescript
// Removido
- selectedChannelListId
- isLoadingChannelList
-