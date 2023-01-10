import { Type } from "@angular/core";
import { invoke } from "@tauri-apps/api/tauri";

import { SearchCommand } from "../search/search.service";
import { DictComponent } from "./dict/dict.component";

export interface Word {
  id: number;
  word: string;
  phonetic: string | undefined;
  definition: Array<string>;
  translation: Array<string>;
  pos: Array<string>;
  collins: number | undefined;
  oxford: number | undefined;
  tag: Array<string>;
  bnc: number | undefined;
  frq: number | undefined;
  exchange: Array<string>;
  detail: string | undefined;
  audio: string | undefined;
}

function map_word_tag(tag: string): string {
  switch (tag) {
    case 'zk':
      return '中考';
    case 'gk':
      return '高考';
    case 'cet4':
      return '四级';
    default:
      return tag;
  }
}

export function gen_markdown(word: Word): string {
  return `
## ${word.word}
${word.phonetic}

${word.exchange.map(v => v).join('/')}

${word.tag.map(v => map_word_tag(v)).join('/')}

${'★'.repeat(word.collins || 0)}${word.oxford ? '※' : ''}

${word.bnc ? `BNC:${word.bnc}` : ''}

${word.frq ? `COCA:${word.frq}` : ''}

${word.pos.map(v => v).join('/')}

### Translation
${word.translation.map(v => `- ${v}`).join('\n')}
### Definition
${word.definition.map(v => `- ${v}`).join('\n')}

`
}

export class DictCommand implements SearchCommand {
  keys(): Array<string> {
    return ['d', 'dict'];
  }

  description(): string {
    return 'dict';
  }

  view(): Type<any> {
    return DictComponent
  }

  async search(text: string): Promise<any> {
    return await invoke('plugin:dict|query', { text, limit: 10 });
  }
}