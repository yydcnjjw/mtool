import { Type } from "@angular/core";
import { invoke } from "@tauri-apps/api/tauri";

import { SearchCommand } from "../search/search.service";
import { DictComponent } from "./dict/dict.component";

export interface WordDetail {
  word: string,
  detail: string,
}

export class DictCommand implements SearchCommand {
  key(): string {
    return 'dict';
  }

  description(): string {
    return 'dict';
  }

  view(): Type<any> {
    return DictComponent
  }

  async search(input: string): Promise<any> {
    return invoke('plugin:dict|query', { input });
  }
}
