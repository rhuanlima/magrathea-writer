# 🌐 Como Contribuir com um Novo Idioma no Magrathea Writer

O **Magrathea Writer** foi projetado com uma arquitetura de internacionalização (i18n) 100% tipada, modular e soberana. Você pode facilmente adicionar suporte a novas línguas e dialetos para autores do mundo todo!

---

## 🛠️ Passo a Passo para Adicionar um Novo Idioma

### 1. Duplique o arquivo base de mensagens
Na pasta `src/i18n/locales/`, escolha um arquivo base para traduzir:
- Português: `src/i18n/locales/pt-BR.ts`
- Inglês: `src/i18n/locales/en-US.ts`

Crie o seu novo arquivo com a convenção BCP-47. Por exemplo, para Espanhol:
```bash
cp src/i18n/locales/en-US.ts src/i18n/locales/es-ES.ts
```
Ou para Francês:
```bash
cp src/i18n/locales/en-US.ts src/i18n/locales/fr-FR.ts
```

### 2. Traduza as chaves mantendo a assinatura tipada
Abra o seu novo arquivo (ex: `src/i18n/locales/es-ES.ts`) e traduza as strings literárias preservando as variáveis interpoladas entre chaves (como `{count}`, `{title}`, `{shortcut}`):

```ts
import type { TranslationSchema } from '../types';

export const esES: TranslationSchema = {
  app: {
    title: 'Magrathea Writer',
    tagline: 'Taller soberano de creación de mundos literarios',
  },
  // ... continue traduzindo as seções
};
```

O compilador TypeScript garante que nenhuma chave obrigatória seja esquecida!

### 3. Registre o novo idioma no `src/i18n/index.ts`
1. Importe o seu novo dicionário:
   ```ts
   import { esES } from './locales/es-ES';
   ```
2. Adicione ao mapa de locales e ao tipo `SupportedLocale`:
   ```ts
   export type SupportedLocale = 'pt-BR' | 'en-US' | 'es-ES';

   const dictionaries: Record<SupportedLocale, TranslationSchema> = {
     'pt-BR': ptBR,
     'en-US': enUS,
     'es-ES': esES,
   };
   ```
3. Adicione o botão de seleção do novo idioma na tela de Preferências (`src/preferences/PreferencesModal.svelte`).

### 4. Teste e Valide
Execute os testes unitários e o linter para assegurar integridade completa:
```bash
npm run typecheck
npm run lint
npm test
```

### 5. Abra um Pull Request
Envie suas alterações em uma nova branch e abra um Pull Request no repositório oficial:
👉 [https://github.com/rhuanlima/magrathea-writer/pulls](https://github.com/rhuanlima/magrathea-writer/pulls)

A comunidade de autores agradece a sua contribuição para tornar o Magrathea acessível em mais idiomas! 🌍✨
