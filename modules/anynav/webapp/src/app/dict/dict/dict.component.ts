import { Component, ElementRef, Input, OnDestroy, OnInit, ViewChild } from '@angular/core';
import hotkeys from 'hotkeys-js';
import { Word } from '../command';

@Component({
  selector: 'app-dict',
  templateUrl: './dict.component.html',
  styleUrls: ['./dict.component.scss']
})
export class DictComponent implements OnInit, OnDestroy {

  @Input()
  data!: Word;

  @ViewChild('view')
  view!: ElementRef<HTMLDivElement>;

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

    hotkeys('alt+o,a', {
      scope: 'dict_search_view',
      keyup: true,
      keydown: true,
    }, (e, keyev) => {
      console.log(e, keyev);
      this.view.nativeElement.focus();
      switch (keyev.key) {
        case 'alt+o':
          this.action = true;
          return;
        default:
          break;
      }

      if (this.action) {
        switch (keyev.key) {
          case 'a':
            this.anki();
            break;
        }
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

}
