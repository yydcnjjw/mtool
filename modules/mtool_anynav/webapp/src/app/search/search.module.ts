import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { SearchDirective, SearchService } from './search.service';

@NgModule({
  declarations: [
    SearchDirective
  ],
  imports: [
    CommonModule
  ],
  exports: [
    SearchDirective
  ]
})
export class SearchModule { }
