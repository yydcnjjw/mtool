import { AfterViewInit, Component, ElementRef, OnDestroy, OnInit, ViewChild } from '@angular/core';
import { globalShortcut } from '@tauri-apps/api';

import { appWindow } from '@tauri-apps/api/window';
import hotkeys from 'hotkeys-js';
import { SearchDirective, SearchService, SearchView } from './search/search.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit, AfterViewInit, OnDestroy {
  title = 'anynav';

  search_placeholder = 'Search: ';

  @ViewChild('box')
  box_view!: ElementRef<HTMLDivElement>;

  @ViewChild('search')
  search_input_view!: ElementRef<HTMLInputElement>;

  @ViewChild(SearchDirective, { static: true }) search_view!: SearchDirective;

  constructor(
    private search_service: SearchService) {
  }

  ngOnInit(): void {
    hotkeys.filter = () => {
      return true;
    }
    hotkeys('ctrl+f,ctrl+b,ctrl+a,ctrl+e,alt+f,alt+b,ctrl+k,alt+e', (e, keyev) => {
      switch (keyev.key) {
        case 'ctrl+f':
          this.forward_char();
          break;
        case 'ctrl+b':
          this.backward_char();
          break;
        case 'ctrl+a':
          e.preventDefault();
          this.move_to_line_begin();
          break;
        case 'ctrl+e':
          this.move_to_line_end();
          break;
        case 'alt+f':
          this.forward_word();
          break;
        case 'alt+b':
          this.backward_word();
          break;
        case 'ctrl+k':
          this.kill();
          break
        case 'alt+e':
          this.focus_input_view();
          break;
        default:
          break;
      }
    });
  }

  ngOnDestroy(): void {
  }

  ngAfterViewInit(): void {
    this.adjust_window_size();
    this.focus_input_view();
  }

  async adjust_window_size() {
    const size = await appWindow.innerSize();

    // size.height = this.box_view.nativeElement.clientHeight * 6;
    size.height = 600;
    await appWindow.setSize(size);
  }

  async toggle_search_view(input: string) {
    const result = await this.search_service.search(input);

    if (!result) {
      return;
    }

    const view = this.search_view.viewContainerRef;
    view.clear();

    const ref = view.createComponent<SearchView>(result.cmd.view()).instance;
    ref.data = result.data;
  }

  move_to_line_begin() {
    this.input_view().setSelectionRange(0, 0);
  }

  move_to_line_end() {
    const len = this.input_view().value.length;
    this.input_view().setSelectionRange(len, len);
  }

  backward_char() {
    const start = this.input_view().selectionStart! - 1;
    const end = this.input_view().selectionEnd! - 1;

    this.input_view().setSelectionRange(start, end);
  }

  forward_char() {
    const start = this.input_view().selectionStart! + 1;
    const end = this.input_view().selectionEnd! + 1;

    this.input_view().setSelectionRange(start, end);
  }

  backward_word() {
    const start = this.input_view().selectionStart! - 1;
    const end = this.input_view().selectionEnd! - 1;

    this.input_view().setSelectionRange(start, end);
  }

  forward_word() {
    const start = this.input_view().selectionStart! + 1;
    const end = this.input_view().selectionEnd! + 1;

    this.input_view().setSelectionRange(start, end);
  }

  kill() {
    this.input_view().value = '';
  }

  focus_input_view() {
    this.input_view().focus();
  }

  input_view() {
    return this.search_input_view.nativeElement;
  }

  async on_search_input(e: Event) {
    const search = e.target as HTMLInputElement;
    const input = search.value;

    await this.toggle_search_view(input);
  }
}
