import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

import { MatDividerModule } from '@angular/material/divider';

import { SearchModule } from './search/search.module';
import { DictModule } from './dict/dict.module';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';


@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserModule,
    AppRoutingModule,
    BrowserAnimationsModule,

    MatDividerModule,

    SearchModule,
    DictModule,
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule {

}