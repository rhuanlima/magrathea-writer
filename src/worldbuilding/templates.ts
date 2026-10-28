/**
 * Definições e geradores dos templates de criação de mundo e fichas guiadas (M4-01 e M4-02).
 */

import type { TranslationKey } from '../i18n';

export type EntityKind = 'character' | 'location' | 'item' | 'faction' | 'lore';

export interface EntityTypeOption {
  kind: EntityKind;
  icon: string;
  labelKey: TranslationKey;
  descKey: TranslationKey;
}

export const ENTITY_TYPE_OPTIONS: EntityTypeOption[] = [
  {
    kind: 'character',
    icon: '👤',
    labelKey: 'worldbuilding.types.character',
    descKey: 'worldbuilding.types.characterDesc',
  },
  {
    kind: 'location',
    icon: '📍',
    labelKey: 'worldbuilding.types.location',
    descKey: 'worldbuilding.types.locationDesc',
  },
  {
    kind: 'item',
    icon: '⚔️',
    labelKey: 'worldbuilding.types.item',
    descKey: 'worldbuilding.types.itemDesc',
  },
  {
    kind: 'faction',
    icon: '🛡️',
    labelKey: 'worldbuilding.types.faction',
    descKey: 'worldbuilding.types.factionDesc',
  },
  {
    kind: 'lore',
    icon: '📜',
    labelKey: 'worldbuilding.types.lore',
    descKey: 'worldbuilding.types.loreDesc',
  },
];
