import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { MatDividerModule } from '@angular/material/divider';
import { MatListModule } from '@angular/material/list';
import { ScrollingModule } from '@angular/cdk/scrolling';

import { SearchModule } from '../search/search.module';
import { SearchService } from '../search/search.service';
import { DictCommand } from './command';
import { DictComponent } from './dict/dict.component';

@NgModule({
  declarations: [
    DictComponent
  ],
  imports: [
    CommonModule,
    SearchModule,

    MatDividerModule,
    MatListModule,
    ScrollingModule
  ],
  providers: [

  ]
})
export class DictModule {
  constructor(
    private search_service: SearchService
  ) {
    this.search_service.add_search_command(new DictCommand)
  }
}
