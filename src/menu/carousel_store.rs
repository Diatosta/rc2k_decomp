//! The menu carousel: the front end sets it up and turns it, the scene draws it and chooses its
//! pictures and textures. Plain fields behind one `RefCell` (the game's thread alone reads and
//! writes them, each borrow lasting one statement), owned by `Screens`.

use crate::memory::session::GameThread;
use crate::menu::lists_store::CarList;
use std::cell::{Ref, RefCell, RefMut};

/// The menu carousel the front end keeps and the scene draws: the entry shown, the list, the
/// turn, the place of its entries, their kinds, and the menu's own numbers.
pub struct CarouselData {
    /// The kind of the entry being drawn.
    pub kind: u32,
    /// The menu's list of cars, `CarList::code` (0: none); `Carousel::list`.
    pub entries: u32,
    /// 1 after right, -1 after left, 0 at rest.
    pub direction: u32,
    /// The x spacing of the places.
    pub spacing: u32,
    /// The y of the places.
    pub y: u32,
    /// The z of the places.
    pub z: u32,
    /// The way forward (the front entry's step).
    pub forward: u32,
    /// The entries' kinds (room for 0x100: the carousel checks that many).
    pub kinds: [u32; 256],
    /// The menu whose carousel is up.
    pub menu: u32,
    /// The entries.
    pub count: u32,
    /// The entry in front.
    pub front: u32,
    /// The turn's angle (0 after right, -1 after left).
    pub angle: u32,
    /// The clock at the last `menu::step_carousel`.
    pub last_clock: u32,
    /// The time since, held to 100.
    pub passed: u32,
    /// Set to 1 by `scene::set_up_carousel` when the carousel's set-up is done; only the
    /// carousel set-up lists (`scene::carousel_set_ups`) name it.
    pub set_up: u32,
    /// The carousel's pictures: four wanted, four shown, four held.
    pub pictures: CarouselPictures,
    /// The eight texture numbers of the carousel's menu (-1 none).
    pub textures: [u32; 8],
}

/// The carousel's pictures: four wanted, four shown, four held.
pub struct CarouselPictures {
    /// The pictures wanted (`scene::draw_car_object`).
    pub wanted: [u32; 4],
    /// The pictures shown.
    pub shown: [u32; 4],
    /// The pictures held (`menu::clear_choice_marks` sets them to -1).
    pub held: [u32; 4],
}

/// The menu carousel.
pub struct Carousel(GameThread<RefCell<CarouselData>>);

impl Carousel {
    pub const fn new() -> Self {
        Carousel(GameThread::new(RefCell::new(CarouselData {
            kind: 0,
            entries: 0,
            direction: 0,
            spacing: 0,
            y: 0,
            z: 0,
            forward: 0,
            kinds: [0; 256],
            menu: 0,
            count: 4,
            front: 3,
            angle: 0,
            last_clock: 0,
            passed: 0,
            set_up: 1,
            pictures: CarouselPictures {
                wanted: [0; 4],
                shown: [0; 4],
                held: [0; 4],
            },
            textures: [0; 8],
        })))
    }

    /// The fields, for reading (one statement).
    pub fn read(&self) -> Ref<'_, CarouselData> {
        self.0.get().borrow()
    }

    /// The fields, for writing (one statement).
    pub fn write(&self) -> RefMut<'_, CarouselData> {
        self.0.get().borrow_mut()
    }

    /// The list the carousel shows, none before one is set up.
    pub fn list(&self) -> Result<Option<CarList>, crate::errors::BadInput> {
        let code = self.read().entries;
        Ok(CarList::from_code(code)?)
    }

    pub fn set_list(&self, list: CarList) {
        self.write().entries = list.code();
    }
}
