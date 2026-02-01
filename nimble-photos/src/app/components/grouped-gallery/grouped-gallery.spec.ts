import { ComponentFixture, TestBed } from '@angular/core/testing';

import { GroupedGallery } from './grouped-gallery';

describe('GroupedGallery', () => {
  let component: GroupedGallery;
  let fixture: ComponentFixture<GroupedGallery>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GroupedGallery]
    })
    .compileComponents();

    fixture = TestBed.createComponent(GroupedGallery);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
