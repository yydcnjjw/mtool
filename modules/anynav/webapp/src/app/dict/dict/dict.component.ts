import { Component, ElementRef, Input, OnDestroy, OnInit, ViewChild } from '@angular/core';
import { MatSelectionList } from '@angular/material/list';
import hotkeys from 'hotkeys-js';
import { Word } from '../command';

@Component({
  selector: 'app-dict',
  templateUrl: './dict.component.html',
  styleUrls: ['./dict.component.scss']
})
export class DictComponent implements OnInit, OnDestroy {

  @Input()
  data!: Array<Word>;

  @ViewChild('view')
  view!: ElementRef<HTMLDivElement>;

  @ViewChild('word_list')
  word_list_view: MatSelectionList | undefined;

  selected_word: Word | undefined;

  action = false;

  map_tag(tag: string): string {
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

  constructor(
  ) { }

  ngOnInit(): void {

    hotkeys('alt+o,a,ctrl+n,ctrl+p,enter', {
      scope: 'dict_search_view',
    }, (e, keyev) => {
      switch (keyev.key) {
        case 'alt+o':
          this.view.nativeElement.focus();
          this.action = true;
          return;
        case 'ctrl+n':
          this.select_next_item();
          break;
        case 'ctrl+p':
          this.select_prev_item();
          break;
        case 'enter':
          this.view_item();
          break;
        default:
          break;
      }

      if (this.action) {
        switch (keyev.key) {
          case 'a':
            this.anki();
            break;
        }

        this.action = false;
      }
    })

    hotkeys.setScope('dict_search_view');
  }

  ngOnDestroy(): void {
    hotkeys.deleteScope('dict_search_view');
  }

  anki() {
    console.log('anki');
  }

  select_next_item() {
    const view = this.word_list_view;
    if (!view) {
      return;
    }

    if (view.selectedOptions.isEmpty()) {
      view.selectedOptions.select(view.options.first)
    } else {
      let index = view.options.toArray().indexOf(view.selectedOptions.selected[0]);
      if (index == view.options.length - 1) {
        index = 0;
      } else {
        index = index + 1;
      }
      view.selectedOptions.select(view.options.get(index)!);
    }
    view.focus({ preventScroll: true });
  }

  select_prev_item() {
    const view = this.word_list_view;
    if (!view) {
      return;
    }

    if (view.selectedOptions.isEmpty()) {
      view.selectedOptions.select(view.options.first)
    } else {
      let index = view.options.toArray().indexOf(view.selectedOptions.selected[0]);
      if (index == 0) {
        index = view.options.length - 1;
      } else {
        index = index - 1;
      }
      view.selectedOptions.select(view.options.get(index)!);

    }
    view.focus({ preventScroll: true });
  }

  view_item() {
    const view = this.word_list_view;
    if (!view) {
      return;
    }

    if (view.selectedOptions.isEmpty()) {
      return;
    }

    this.selected_word = view.selectedOptions.selected[0].value;
  }

}
