import { AfterViewInit, Component, ElementRef, OnInit, ViewChild } from '@angular/core';

import { appWindow } from '@tauri-apps/api/window';
import { SearchDirective, SearchService, SearchView } from './search/search.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit, AfterViewInit {
  title = 'anynav';

  search_placeholder = 'Search: ';

  @ViewChild('box')
  box_view!: ElementRef<HTMLDivElement>;

  @ViewChild(SearchDirective, { static: true }) search_view!: SearchDirective;

  constructor(
    private search_service: SearchService) {
  }

  ngOnInit(): void {

  }

  ngAfterViewInit(): void {
    this.adjust_window_size();
  }

  async adjust_window_size() {
    const size = await appWindow.innerSize();

    // size.height = this.box_view.nativeElement.clientHeight * 6;
    size.height = 600;
    await appWindow.setSize(size);
  }

  async on_search_input(e: Event) {
    const search = e.target as HTMLInputElement;
    const input = search.value;

    const result = await this.search_service.search(input);

    if (!result) {
      return;
    }

    const view = this.search_view.viewContainerRef;
    view.clear();

    const ref = view.createComponent<SearchView>(result.cmd.view()).instance;
    ref.data = result.data;
  }
}
