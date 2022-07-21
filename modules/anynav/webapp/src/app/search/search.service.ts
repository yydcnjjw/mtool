import { Directive, Injectable, Type, ViewContainerRef } from '@angular/core';

export interface SearchView {
  data: any;
};

@Directive({
  selector: '[search-view]',
})
export class SearchDirective {
  constructor(public viewContainerRef: ViewContainerRef) { }
}

export interface SearchCommand {
  keys(): Array<string>;
  description(): string;
  view(): Type<any>;

  search(input: string): Promise<any>;
};

export interface SearchResult {
  cmd: SearchCommand;
  data: any;
};

@Injectable({
  providedIn: 'root'
})
export class SearchService {

  cmds = new Map<string, SearchCommand>();

  constructor() { }

  add_search_command(command: SearchCommand) {
    for (let key of command.keys()) {
      this.cmds.set(key, command);
    }
  }

  async search(input: string): Promise<SearchResult | undefined> {
    const splits = input.split(':');
    const key = splits.at(0)!;
    const text = splits.at(1) || '';

    if (!this.cmds.has(key)) {
      return undefined;
    }

    const cmd = this.cmds.get(key)!;
    try {
      return { cmd, data: await cmd.search(text.trim()) };
    } catch (e) {
      console.log(`${key} search error: ${e}`)
      return undefined;
    }
  }
}
