import { Component, Input, OnInit } from '@angular/core';
import { WordDetail } from '../command';

@Component({
  selector: 'app-dict',
  templateUrl: './dict.component.html',
  styleUrls: ['./dict.component.scss']
})
export class DictComponent implements OnInit {

  @Input()
  data!: WordDetail;

  constructor() { }

  ngOnInit(): void {

    console.log(this.data);
    
  }

}
