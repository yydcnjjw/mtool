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
