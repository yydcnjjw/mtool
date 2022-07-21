import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { MatDividerModule } from '@angular/material/divider';

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
    MatDividerModule
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
